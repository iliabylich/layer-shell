use crate::{
    FFIArray,
    utils::{StringRef, StringRefExt as _},
};
use anyhow::{Context as _, Result};
use boml::{Toml, table::TomlTable, types::TomlArray};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) struct Config {
    pub(crate) lock: String,
    pub(crate) reboot: String,
    pub(crate) shutdown: String,
    pub(crate) logout: String,
    pub(crate) edit_wifi: String,
    pub(crate) edit_bluetooth: String,
    pub(crate) open_system_monitor: String,
    pub(crate) change_wallpaper: String,

    pub(crate) ping: Vec<String>,
    pub(crate) terminal: Terminal,
}
#[derive(Debug)]
pub(crate) struct Terminal {
    label: String,
    command: Vec<String>,
}
impl Terminal {
    fn from_toml(toml: &TomlTable<'_>) -> Result<Self> {
        let label = toml
            .get_string("label")
            .map_err(|err| anyhow::anyhow!("{err:?}"))?
            .to_string();
        let command = toml
            .get_array("command")
            .map_err(|err| anyhow::anyhow!("{err:?}"))?;
        Ok(Self {
            label,
            command: toml_value_to_array_of_strings(command)?,
        })
    }
}

fn toml_value_to_array_of_strings(toml: &TomlArray<'_>) -> Result<Vec<String>> {
    toml.iter()
        .map(|e| {
            e.as_string()
                .context("array item is not a string")
                .map(ToString::to_string)
        })
        .collect()
}

impl Config {
    pub(crate) fn read() -> Result<Self> {
        let path = config_dir()?.join("layer-shell").join("config.toml");
        let contents = std::fs::read_to_string(&path)?;
        let toml = boml::parse(&contents).map_err(|err| anyhow::anyhow!("{err}"))?;
        let config = Self::from_toml(&toml)?;

        log::info!(target: "Config", "{config:#?}");

        Ok(config)
    }

    fn from_toml(toml: &Toml<'_>) -> Result<Self> {
        macro_rules! string {
            ($key:expr) => {
                toml.get($key)
                    .with_context(|| format!("no {}", $key))?
                    .as_string()
                    .with_context(|| format!("{} is not a string", $key))?
                    .to_string()
            };
        }
        let lock = string!("lock");
        let reboot = string!("reboot");
        let shutdown = string!("shutdown");
        let logout = string!("logout");
        let edit_wifi = string!("edit_wifi");
        let edit_bluetooth = string!("edit_bluetooth");
        let open_system_monitor = string!("open_system_monitor");
        let change_wallpaper = string!("change_wallpaper");
        let ping = toml
            .get_array("ping")
            .map_err(|err| anyhow::anyhow!("{err:?}"))?;
        let terminal = toml
            .get_table("terminal")
            .map_err(|err| anyhow::anyhow!("{err:?}"))?;

        Ok(Self {
            lock,
            reboot,
            shutdown,
            logout,
            edit_wifi,
            edit_bluetooth,
            open_system_monitor,
            change_wallpaper,
            ping: toml_value_to_array_of_strings(ping)?,
            terminal: Terminal::from_toml(terminal)?,
        })
    }
}

fn config_dir() -> Result<PathBuf> {
    if let Ok(xdg_config_dir) = std::env::var("XDG_CONFIG_DIR") {
        Ok(PathBuf::from(xdg_config_dir))
    } else {
        let home = std::env::var("HOME").context("no $HOME")?;
        Ok(Path::new(&home).join(".config"))
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct IOConfig {
    pub ping: FFIArray<StringRef>,
    pub terminal: IOTerminal,
}
impl IOConfig {
    pub(crate) fn new(config: &Config) -> Self {
        Self {
            ping: config
                .ping
                .iter()
                .map(|s| StringRef::new(s))
                .collect::<Vec<_>>()
                .into(),
            terminal: IOTerminal {
                label: StringRef::new(&config.terminal.label),
                command: config
                    .terminal
                    .command
                    .iter()
                    .map(|s| StringRef::new(s))
                    .collect::<Vec<_>>()
                    .into(),
            },
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct IOTerminal {
    pub label: StringRef,
    pub command: FFIArray<StringRef>,
}
