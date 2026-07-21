use crate::{
    FixedSizeArrray,
    utils::{ArrayWriter, StringRef, StringRefExt as _, getenv},
};
use anyhow::{Context as _, Result, bail, ensure};
use core::fmt::Write;
use libc::{O_RDONLY, close, open, read};

#[derive(Debug)]
pub(crate) struct Config {
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
pub(crate) struct Terminal {
    pub(crate) label: StringRef,
    pub(crate) command: FixedSizeArrray<10, StringRef>,
}

impl Config {
    pub(crate) fn read() -> Result<Self> {
        let mut buf = [0; 256];
        let mut writer = ArrayWriter::new(&mut buf);
        fmt_config_path(&mut writer)?;
        let path = writer.as_str()?;

        let fd = unsafe { open(path.as_ptr().cast(), O_RDONLY) };
        ensure!(fd != -1, "failed to open config at {path:?}");
        let fd = AutoCloseFd(fd);

        let mut buf = [0; 1_024];
        let res = unsafe { read(fd.0, buf.as_mut_ptr().cast(), 1_024) };

        let len = usize::try_from(res).context("failed to read config")?;
        let buf = buf
            .get(..len)
            .context("reading config exceeded buffer size")?;

        let contents = core::str::from_utf8(buf).context("non-utf8 config")?;
        let config = Self::from_toml(contents)?;

        log::info!(target: "Config", "{config:#?}");

        Ok(config)
    }

    fn from_toml(contents: &str) -> Result<Self> {
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
            let (key, value) = line.split_once('=').context("malformed line")?;
            let key = key.trim();
            let value = value
                .trim()
                .strip_prefix('"')
                .context("value doesn't have \" prefix")?
                .strip_suffix('"')
                .context("value doesn't have \" suffix")?;

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
                _ => bail!("unknown config key {key}"),
            }
        }

        let lock = lock.context("no lock")?;
        let reboot = reboot.context("no reboot")?;
        let shutdown = shutdown.context("no shutdown")?;
        let logout = logout.context("no logout")?;
        let edit_wifi = edit_wifi.context("no edit_wifi")?;
        let edit_bluetooth = edit_bluetooth.context("no edit_bluetooth")?;
        let open_system_monitor = open_system_monitor.context("no open_system_monitor")?;
        let change_wallpaper = change_wallpaper.context("no change_wallpaper")?;
        let ping = ping.context("no ping")?;
        let terminal_label = terminal_label.context("no terminal_label")?;
        let terminal_command = terminal_command.context("no terminal_command")?;

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
                let mut out = FixedSizeArrray::empty_with_default_fn(|| StringRef::new(""));
                for part in ping.split_whitespace() {
                    let part = StringRef::new(part);
                    out.push(part)?;
                }
                out
            },
            terminal: Terminal {
                label: StringRef::new(terminal_label),
                command: {
                    let mut out = FixedSizeArrray::empty_with_default_fn(|| StringRef::new(""));
                    for part in terminal_command.split_whitespace() {
                        let part = StringRef::new(part);
                        out.push(part)?;
                    }
                    out
                },
            },
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

struct AutoCloseFd(i32);

impl Drop for AutoCloseFd {
    fn drop(&mut self) {
        unsafe { close(self.0) };
    }
}
