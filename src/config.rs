use crate::{
    FFIArray,
    utils::{ArrayWriter, StringRef, StringRefExt as _, getenv},
};
use anyhow::{Context as _, Result, ensure};
use boml::{Toml, table::TomlTable, types::TomlArray};
use core::fmt::Write;

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
        let mut buf = [0; 256];
        let mut writer = ArrayWriter::new(&mut buf);
        fmt_config_path(&mut writer)?;
        let path = writer.as_str()?;

        let fd = unsafe { libc::open(path.as_ptr().cast(), libc::O_RDONLY) };
        ensure!(fd != -1, "failed to open config at {path:?}");
        let fd = AutoCloseFd(fd);

        let mut buf = [0; 1_024];
        let res = unsafe { libc::read(fd.0, buf.as_mut_ptr().cast(), 1_024) };

        let len = usize::try_from(res).context("failed to read config")?;
        let buf = buf
            .get(..len)
            .context("reading config exceeded buffer size")?;

        let contents = core::str::from_utf8(buf).context("non-utf8 config")?;
        let toml = boml::parse(contents).map_err(|err| anyhow::anyhow!("{err}"))?;
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

fn fmt_config_path(mut w: impl Write) -> Result<()> {
    let xdg_config_dir = getenv(c"XDG_CONFIG_DIR")
        .map(core::str::from_utf8)
        .transpose()
        .context("non-utf8 $XDG_CONFIG_DIR")?;
    let home = getenv(c"HOME")
        .map(core::str::from_utf8)
        .transpose()
        .context("non-utf8 $HOME")?
        .context("no $HOME")?;

    if let Some(xdg_config_dir) = xdg_config_dir {
        write!(&mut w, "{xdg_config_dir}/layer-shell/config.toml")?;
    } else {
        write!(&mut w, "{home}/.config/layer-shell/config.toml")?;
    }
    Ok(())
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

struct AutoCloseFd(i32);

impl Drop for AutoCloseFd {
    fn drop(&mut self) {
        unsafe { libc::close(self.0) };
    }
}
