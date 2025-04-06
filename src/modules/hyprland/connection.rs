use anyhow::{Context as _, Result};
use mio::net::UnixStream;

pub(crate) fn connect_to_socket() -> Result<UnixStream> {
    let path = format!(
        "{}/hypr/{}/.socket2.sock",
        xdg_runtime_dir()?,
        hyprland_instance_signature()?
    );
    let socket = UnixStream::connect(&path).context("failed to open reader socket")?;
    Ok(socket)
}

fn xdg_runtime_dir() -> Result<String> {
    std::env::var("XDG_RUNTIME_DIR").context("no XDG_RUNTIME_DIR variable")
}

fn hyprland_instance_signature() -> Result<String> {
    std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .context("no HYPRLAND_INSTANCE_SIGNATURE, are you in Hyprland?")
}
