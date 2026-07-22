use crate::{
    FixedSizeArrray,
    error::IoError,
    utils::{StringRef, StringRefExt as _, log_err_and_exit, write_in_place},
};
use rustix::fs::{Mode, OFlags};
use thiserror::Error;

#[derive(Debug)]
pub struct Config {
    pub(crate) lock: StringRef,
    pub(crate) reboot: StringRef,
    pub(crate) shutdown: StringRef,
    pub(crate) logout: StringRef,
    pub(crate) edit_wifi: StringRef,
    pub(crate) edit_bluetooth: StringRef,
    pub(crate) open_system_monitor: StringRef,
    pub(crate) change_wallpaper: StringRef,

    pub(crate) ping: FixedSizeArrray<10, StringRef>,
    pub(crate) terminal: Terminal,
}
#[derive(Debug)]
pub struct Terminal {
    pub(crate) label: StringRef,
    pub(crate) command: FixedSizeArrray<10, StringRef>,
}

impl Config {
    pub(crate) fn read(xdg_config_dir: Option<&str>, home: &str) -> Self {
        match Self::try_read(xdg_config_dir, home) {
            Ok(config) => config,
            Err(err) => log_err_and_exit!("{err:?}"),
        }
    }

    fn try_read(xdg_config_dir: Option<&str>, home: &str) -> Result<Self, IoError> {
        let mut buf = [0; 256];
        let path = if let Some(xdg_config_dir) = xdg_config_dir {
            write_in_place!(&mut buf, "{xdg_config_dir}/layer-shell/config.toml")
        } else {
            write_in_place!(&mut buf, "{home}/.config/layer-shell/config.toml")
        };

        let fd = rustix::fs::open(path, OFlags::RDONLY, Mode::empty())
            .map_err(|errno| IoError::FailedTo { op: "open", errno })?;

        let mut buf = [0; 1_024];
        let len = rustix::io::read(&fd, &mut buf)
            .map_err(|errno| IoError::FailedTo { op: "read", errno })?;
        let buf = buf
            .get(..len)
            .unwrap_or_else(|| log_err_and_exit!("read failed"));

        let contents = core::str::from_utf8(buf).map_err(ConfigError::NonUtf8Config)?;
        let config = Self::from_toml(contents)?;

        log::info!(target: "Config", "{config:#?}");

        Ok(config)
    }

    fn from_toml(contents: &str) -> Result<Self, ConfigError> {
        let mut lock = None;
        let mut reboot = None;
        let mut shutdown = None;
        let mut logout = None;
        let mut edit_wifi = None;
        let mut edit_bluetooth = None;
        let mut open_system_monitor = None;
        let mut change_wallpaper = None;
        let mut ping = None;
        let mut terminal_label = None;
        let mut terminal_command = None;

        for line in contents.lines() {
            let (key, value) = line
                .split_once('=')
                .ok_or(ConfigError::LineHasNoEqDelimiter)?;
            let key = key.trim();
            let value = value
                .trim()
                .strip_prefix('"')
                .ok_or(ConfigError::ValueMissingQuotePrefix)?
                .strip_suffix('"')
                .ok_or(ConfigError::ValueMissingQuoteSuffix)?;

            match key {
                "lock" => lock = Some(value),
                "reboot" => reboot = Some(value),
                "shutdown" => shutdown = Some(value),
                "logout" => logout = Some(value),
                "edit_wifi" => edit_wifi = Some(value),
                "edit_bluetooth" => edit_bluetooth = Some(value),
                "open_system_monitor" => open_system_monitor = Some(value),
                "change_wallpaper" => change_wallpaper = Some(value),
                "ping" => ping = Some(value),
                "terminal_label" => terminal_label = Some(value),
                "terminal_command" => terminal_command = Some(value),
                _ => return Err(ConfigError::UnknownKey),
            }
        }

        let lock = lock.ok_or(ConfigError::MissingKey("lock"))?;
        let reboot = reboot.ok_or(ConfigError::MissingKey("reboot"))?;
        let shutdown = shutdown.ok_or(ConfigError::MissingKey("shutdown"))?;
        let logout = logout.ok_or(ConfigError::MissingKey("logout"))?;
        let edit_wifi = edit_wifi.ok_or(ConfigError::MissingKey("edit_wifi"))?;
        let edit_bluetooth = edit_bluetooth.ok_or(ConfigError::MissingKey("edit_bluetooth"))?;
        let open_system_monitor =
            open_system_monitor.ok_or(ConfigError::MissingKey("open_system_monitor"))?;
        let change_wallpaper =
            change_wallpaper.ok_or(ConfigError::MissingKey("change_wallpaper"))?;
        let ping = ping.ok_or(ConfigError::MissingKey("ping"))?;
        let terminal_label = terminal_label.ok_or(ConfigError::MissingKey("terminal_label"))?;
        let terminal_command =
            terminal_command.ok_or(ConfigError::MissingKey("terminal_command"))?;

        Ok(Self {
            lock: StringRef::new(lock),
            reboot: StringRef::new(reboot),
            shutdown: StringRef::new(shutdown),
            logout: StringRef::new(logout),
            edit_wifi: StringRef::new(edit_wifi),
            edit_bluetooth: StringRef::new(edit_bluetooth),
            open_system_monitor: StringRef::new(open_system_monitor),
            change_wallpaper: StringRef::new(change_wallpaper),
            ping: {
                let mut out = FixedSizeArrray::empty_with_default_fn(StringRef::empty);
                for part in ping.split_whitespace() {
                    let part = StringRef::new(part);
                    out.push(part).ok_or(ConfigError::PingCommandIsTooLong)?;
                }
                out
            },
            terminal: Terminal {
                label: StringRef::new(terminal_label),
                command: {
                    let mut out = FixedSizeArrray::empty_with_default_fn(StringRef::empty);
                    for part in terminal_command.split_whitespace() {
                        let part = StringRef::new(part);
                        out.push(part)
                            .ok_or(ConfigError::TerminalCommandIsTooLong)?;
                    }
                    out
                },
            },
        })
    }
}

#[derive(Debug, Error, Clone, Copy)]
pub enum ConfigError {
    #[error("non-utf8 config")]
    NonUtf8Config(core::str::Utf8Error),

    #[error("malformed config line")]
    LineHasNoEqDelimiter,
    #[error("value does not have quote prefix")]
    ValueMissingQuotePrefix,
    #[error("value does not have quote suffix")]
    ValueMissingQuoteSuffix,
    #[error("unknown config key")]
    UnknownKey,
    #[error("missing config key: {0:?}")]
    MissingKey(&'static str),
    #[error("terminal command is too long")]
    TerminalCommandIsTooLong,
    #[error("ping command is too long")]
    PingCommandIsTooLong,
}
