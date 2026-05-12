use crate::utils::{StringRef, StringRefExt as _};
use anyhow::{Context as _, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug)]
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
#[derive(Deserialize, Debug)]
pub(crate) struct Terminal {
    label: String,
    command: Vec<String>,
}

impl Config {
    pub(crate) fn read() -> Result<Self> {
        let path = config_dir()?.join("layer-shell").join("config.toml");
        let contents = std::fs::read_to_string(&path)?;
        let config = toml::from_str(&contents)?;

        log::info!(target: "Config", "{config:#?}");

        Ok(config)
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

#[repr(C)]
pub struct IOConfig {
    pub ping: *mut *mut core::ffi::c_char,
    pub terminal: IOTerminal,
}
#[repr(C)]
pub struct IOTerminal {
    pub label: StringRef,
    pub command: *mut *mut core::ffi::c_char,
}

impl TryFrom<&Config> for IOConfig {
    type Error = anyhow::Error;

    fn try_from(config: &Config) -> Result<Self> {
        Ok(Self {
            ping: vec_of_string_to_null_terminated_c_array(&config.ping)?,
            terminal: IOTerminal::try_from(&config.terminal)?,
        })
    }
}

impl TryFrom<&Terminal> for IOTerminal {
    type Error = anyhow::Error;

    fn try_from(terminal: &Terminal) -> Result<Self> {
        Ok(Self {
            label: StringRef::new(&terminal.label),
            command: vec_of_string_to_null_terminated_c_array(&terminal.command)?,
        })
    }
}

fn vec_of_string_to_null_terminated_c_array(cmd: &[String]) -> Result<*mut *mut core::ffi::c_char> {
    let mut cmd = cmd
        .iter()
        .map(|s| Ok(std::ffi::CString::new(s.clone().into_bytes())?.into_raw()))
        .collect::<Result<Vec<_>>>()?;
    cmd.push(core::ptr::null_mut());
    let mut cmd = cmd.into_boxed_slice();
    let ptr = cmd.as_mut_ptr();
    core::mem::forget(cmd);
    Ok(ptr)
}
