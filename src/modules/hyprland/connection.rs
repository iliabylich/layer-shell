use anyhow::{Context as _, Result};
use std::os::unix::net::UnixStream;

pub(crate) fn connect_to_socket() -> Result<UnixStream> {
    let socket_path = hyprland_socket_path()?;
    let socket = UnixStream::connect(&socket_path).context("failed to open unix socket")?;
    Ok(socket)
}

fn hyprland_socket_path() -> Result<String> {
    let xdg_runtime_dir =
        std::env::var("XDG_RUNTIME_DIR").context("no XDG_RUNTIME_DIR variable")?;
    let hyprland_instance_signature = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .context("no HYPRLAND_INSTANCE_SIGNATURE, are you in Hyprland?")?;
    Ok(format!(
        "{}/hypr/{}/.socket2.sock",
        xdg_runtime_dir, hyprland_instance_signature
    ))
}
