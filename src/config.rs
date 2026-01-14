use crate::ffi::CString;
use anyhow::{Context as _, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug)]
pub(crate) struct Config {
    pub(crate) lock: String,
    pub(crate) reboot: String,
    pub(crate) shutdown: String,
    pub(crate) edit_wifi: String,
    pub(crate) edit_bluetooth: String,
    pub(crate) open_system_monitor: String,
    pub(crate) change_theme: String,

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
        let contents =
            std::fs::read_to_string(&path).with_context(|| format!("failed to read {path:?}"))?;
        let config: Config =
            toml::from_str(&contents).with_context(|| format!("failed to parse {path:?}"))?;

        log::info!("{config:#?}");

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
    pub lock: CString,
    pub reboot: CString,
    pub shutdown: CString,
    pub edit_wifi: CString,
    pub edit_bluetooth: CString,
    pub open_system_monitor: CString,
    pub change_theme: CString,

    pub ping: *mut *mut std::ffi::c_char,
    pub terminal: IOTerminal,
}
#[repr(C)]
pub struct IOTerminal {
    pub label: CString,
    pub command: *mut *mut std::ffi::c_char,
}

impl From<&Config> for IOConfig {
    fn from(config: &Config) -> Self {
        Self {
            lock: config.lock.clone().into(),
            reboot: config.reboot.clone().into(),
            shutdown: config.shutdown.clone().into(),
            edit_wifi: config.edit_wifi.clone().into(),
            edit_bluetooth: config.edit_bluetooth.clone().into(),
            open_system_monitor: config.open_system_monitor.clone().into(),
            change_theme: config.change_theme.clone().into(),

            ping: vec_of_string_to_null_terminated_c_array(config.ping.clone()),
            terminal: IOTerminal::from(&config.terminal),
        }
    }
}

impl From<&Terminal> for IOTerminal {
    fn from(terminal: &Terminal) -> Self {
        Self {
            label: terminal.label.clone().into(),
            command: vec_of_string_to_null_terminated_c_array(terminal.command.clone()),
        }
    }
}

fn vec_of_string_to_null_terminated_c_array(cmd: Vec<String>) -> *mut *mut std::ffi::c_char {
    let mut cmd = cmd
        .into_iter()
        .map(|s| CString::from(s).into_raw())
        .collect::<Vec<_>>();
    cmd.push(std::ptr::null_mut());
    let mut cmd = cmd.into_boxed_slice();
    let ptr = cmd.as_mut_ptr();
    std::mem::forget(cmd);
    ptr
}
