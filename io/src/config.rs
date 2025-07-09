use anyhow::{Context as _, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
pub(crate) struct Config {
    pub(crate) lock: String,
    pub(crate) reboot: String,
    pub(crate) shutdown: String,
    pub(crate) edit_wifi: String,
    pub(crate) edit_bluetooth: String,
    pub(crate) open_system_monitor: String,
    pub(crate) change_theme: String,
}

impl Config {
    pub(crate) fn read() -> Result<Self> {
        let path = config_dir()?.join("layer-shell").join("config.toml");
        let contents =
            std::fs::read_to_string(&path).with_context(|| format!("failed to read {path:?}"))?;
        let config: Config =
            toml::from_str(&contents).with_context(|| format!("failed to parse {path:?}"))?;

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
    pub lock: ffi::CString,
    pub reboot: ffi::CString,
    pub shutdown: ffi::CString,
    pub edit_wifi: ffi::CString,
    pub edit_bluetooth: ffi::CString,
    pub open_system_monitor: ffi::CString,
    pub change_theme: ffi::CString,
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
        }
    }
}
