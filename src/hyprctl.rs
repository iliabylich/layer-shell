use std::{
    io::{Read as _, Write as _},
    os::unix::net::UnixStream,
};

use anyhow::{Context as _, Result, bail};

pub(crate) fn write(cmd: impl AsRef<str>) -> Result<String> {
    let path = format!(
        "{}/hypr/{}/.socket.sock",
        xdg_runtime_dir()?,
        hyprland_instance_signature()?
    );
    let mut socket = UnixStream::connect(&path).context("failed to open writer socket")?;

    let cmd = cmd.as_ref();
    socket.write_all(cmd.as_bytes())?;

    let mut out = vec![];
    socket
        .read_to_end(&mut out)
        .context("failed to read to end")?;
    let out = String::from_utf8(out).context("non-utf-8 response from hyprland socket")?;

    Ok(out)
}

pub(crate) fn dispatch(cmd: impl AsRef<str>) -> Result<()> {
    let res = write(format!("dispatch {}", cmd.as_ref()))?;
    if res != "ok" {
        bail!(
            "invalid response from hyprctl dispatch: expected 'ok', got {:?}",
            res
        );
    }
    Ok(())
}

fn xdg_runtime_dir() -> Result<String> {
    std::env::var("XDG_RUNTIME_DIR").context("no XDG_RUNTIME_DIR variable")
}

fn hyprland_instance_signature() -> Result<String> {
    std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .context("no HYPRLAND_INSTANCE_SIGNATURE, are you in Hyprland?")
}
