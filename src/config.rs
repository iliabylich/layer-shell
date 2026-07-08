use crate::utils::{ArrayWriter, StringRef, StringRefExt as _, getenv};
use alloc::vec::Vec;
use anyhow::{Context as _, Result, ensure};
use boml::{Toml, table::TomlTable, types::TomlArray};
use core::fmt::Write;

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

    pub(crate) ping: Vec<StringRef>,
    pub(crate) terminal: Terminal,
}
#[derive(Debug)]
pub(crate) struct Terminal {
    pub(crate) label: StringRef,
    pub(crate) command: Vec<StringRef>,
}
impl Terminal {
    fn from_toml(toml: &TomlTable<'_>) -> Result<Self> {
        let label = toml
            .get_string("label")
            .map(StringRef::new)
            .map_err(|err| anyhow::anyhow!("{err:?}"))?;
        let command = toml
            .get_array("command")
            .map_err(|err| anyhow::anyhow!("{err:?}"))?;
        Ok(Self {
            label,
            command: toml_value_to_array_of_strings(command)?,
        })
    }
}

fn toml_value_to_array_of_strings(toml: &TomlArray<'_>) -> Result<Vec<StringRef>> {
    toml.iter()
        .map(|e| {
            e.as_string()
                .context("array item is not a string")
                .map(StringRef::new)
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
                    .context(concat!("no ", $key))?
                    .as_string()
                    .map(StringRef::new)
                    .context(concat!($key, " is not a string"))?
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

struct AutoCloseFd(i32);

impl Drop for AutoCloseFd {
    fn drop(&mut self) {
        unsafe { libc::close(self.0) };
    }
}
