use crate::{
    FixedSizeArrray,
    utils::{StringRef, StringRefExt as _, log_err_and_exit, write_in_place},
};
use rustix::fs::{Mode, OFlags};

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
        let mut buf = [0; 256];
        let path = if let Some(xdg_config_dir) = xdg_config_dir {
            write_in_place!(&mut buf, "{xdg_config_dir}/layer-shell/config.toml")
        } else {
            write_in_place!(&mut buf, "{home}/.config/layer-shell/config.toml")
        };

        let fd = rustix::fs::open(path, OFlags::RDONLY, Mode::empty())
            .unwrap_or_else(|err| log_err_and_exit!("failed to open config: {err:?}"));

        let mut buf = [0; 1_024];
        let len = rustix::io::read(&fd, &mut buf)
            .unwrap_or_else(|err| log_err_and_exit!("failed to read config: {err:?}"));
        let buf = buf
            .get(..len)
            .unwrap_or_else(|| log_err_and_exit!("read failed"));

        let contents = core::str::from_utf8(buf)
            .unwrap_or_else(|err| log_err_and_exit!("non-utf8 input: {err:?}"));
        let config = Self::from_toml(contents);

        log::info!("{config:#?}");

        config
    }

    fn from_toml(contents: &str) -> Self {
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
                .unwrap_or_else(|| log_err_and_exit!("no '=' separator on line {line:?}"));
            let key = key.trim();
            let value = value
                .trim()
                .strip_prefix('"')
                .unwrap_or_else(|| log_err_and_exit!("no leading quote on line {line:?}"))
                .strip_suffix('"')
                .unwrap_or_else(|| log_err_and_exit!("no trailing quote on line {line:?}"));

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
                _ => log_err_and_exit!("unknown key {key}"),
            }
        }

        macro_rules! expect_key {
            ($k:ident) => {
                let $k = $k.unwrap_or_else(|| {
                    log_err_and_exit!("no key {} in the config", stringify!($k))
                });
            };
        }
        expect_key!(lock);
        expect_key!(reboot);
        expect_key!(shutdown);
        expect_key!(logout);
        expect_key!(edit_wifi);
        expect_key!(edit_bluetooth);
        expect_key!(open_system_monitor);
        expect_key!(change_wallpaper);
        expect_key!(ping);
        expect_key!(terminal_label);
        expect_key!(terminal_command);

        Self {
            lock: StringRef::new(lock),
            reboot: StringRef::new(reboot),
            shutdown: StringRef::new(shutdown),
            logout: StringRef::new(logout),
            edit_wifi: StringRef::new(edit_wifi),
            edit_bluetooth: StringRef::new(edit_bluetooth),
            open_system_monitor: StringRef::new(open_system_monitor),
            change_wallpaper: StringRef::new(change_wallpaper),
            ping: split_command(ping),
            terminal: Terminal {
                label: StringRef::new(terminal_label),
                command: split_command(terminal_command),
            },
        }
    }
}

fn split_command(command: &str) -> FixedSizeArrray<10, StringRef> {
    let mut out = FixedSizeArrray::empty_with_default_fn(StringRef::empty);
    for part in command.split_whitespace() {
        let part = StringRef::new(part);
        out.push(part)
            .unwrap_or_else(|| log_err_and_exit!("command is too long: {command:?}"));
    }
    out
}
