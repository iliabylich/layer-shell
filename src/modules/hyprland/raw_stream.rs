use crate::modules::hyprland::raw_event::RawEvent;
use anyhow::{Context as _, Result};
use async_stream::stream;
use futures::Stream;
use tokio::{
    io::{AsyncBufReadExt as _, BufReader},
    net::UnixStream,
};

pub(crate) async fn raw_events_stream() -> impl Stream<Item = RawEvent> {
    stream! {
        match connect_to_socket().await {
            Ok(socket) => {
                let buffered = BufReader::new(socket);
                let mut lines = buffered.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    if let Some(event) = RawEvent::parse(line) {
                        yield event;
                    }
                }
            },
            Err(err) => {
                log::error!("Failed to connect to Hyprland socket: {:?}", err);
            }
        }
    }
}

async fn connect_to_socket() -> Result<UnixStream> {
    let socket_path = hyprland_socket_path()?;
    let socket = UnixStream::connect(&socket_path)
        .await
        .context("failed to open unix socket")?;
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
