use crate::{ffi::ShortString, utils::report_and_exit};
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
    pub label: ShortString,
    pub command: *mut *mut core::ffi::c_char,
}

impl From<&Config> for IOConfig {
    fn from(config: &Config) -> Self {
        Self {
            ping: vec_of_string_to_null_terminated_c_array(&config.ping),
            terminal: IOTerminal::from(&config.terminal),
        }
    }
}

impl From<&Terminal> for IOTerminal {
    fn from(terminal: &Terminal) -> Self {
        Self {
            label: terminal.label.as_str().into(),
            command: vec_of_string_to_null_terminated_c_array(&terminal.command),
        }
    }
}

fn vec_of_string_to_null_terminated_c_array(cmd: &[String]) -> *mut *mut core::ffi::c_char {
    let mut cmd = cmd
        .iter()
        .map(|s| {
            std::ffi::CString::new(s.clone().into_bytes())
                .unwrap_or_else(|err| report_and_exit!("{:?}", err))
                .into_raw()
        })
        .collect::<Vec<_>>();
    cmd.push(core::ptr::null_mut());
    let mut cmd = cmd.into_boxed_slice();
    let ptr = cmd.as_mut_ptr();
    core::mem::forget(cmd);
    ptr
}
