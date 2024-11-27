use crate::raw_event::RawEvent;
use anyhow::{Context as _, Result};
use futures::Stream;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    io::{AsyncBufReadExt as _, BufReader, Lines},
    net::UnixStream,
};

pub(crate) struct RawStream {
    inner: Lines<BufReader<UnixStream>>,
}

impl RawStream {
    pub(crate) async fn new() -> Result<Self> {
        let socket_path = hyprland_socket_path()?;
        let socket = UnixStream::connect(&socket_path)
            .await
            .context("failed to open unix socket")?;
        let buffered = BufReader::new(socket);
        let lines = buffered.lines();

        Ok(Self { inner: lines })
    }
}

impl Stream for RawStream {
    type Item = RawEvent;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner).poll_next_line(cx) {
            Poll::Ready(Err(err)) => {
                log::error!("{:?}", err);
                Poll::Ready(None)
            }
            Poll::Ready(Ok(None)) => {
                log::error!("empty line from Hyprland socket");
                Poll::Ready(None)
            }
            Poll::Ready(Ok(Some(line))) => match RawEvent::parse(line) {
                Some(event) => Poll::Ready(Some(event)),
                None => Poll::Pending,
            },
            Poll::Pending => Poll::Pending,
        }
    }
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
