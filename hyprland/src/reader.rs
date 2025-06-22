use crate::env::{hyprland_instance_signature, xdg_runtime_dir};
use anyhow::{Context as _, Result};
use futures_util::{Stream, ready};
use pin_project_lite::pin_project;
use std::task::Poll::*;
use tokio::{
    io::{AsyncBufReadExt, BufReader, Lines},
    net::UnixStream,
};

pin_project! {
    pub(crate) struct Reader {
        #[pin]
        socket: Lines<BufReader<UnixStream>>,
    }
}

impl Reader {
    pub(crate) async fn new() -> Result<Self> {
        let path = format!(
            "{}/hypr/{}/.socket2.sock",
            xdg_runtime_dir()?,
            hyprland_instance_signature()?
        );
        let socket = UnixStream::connect(&path)
            .await
            .context("failed to open reader socket")?;
        Ok(Self {
            socket: BufReader::new(socket).lines(),
        })
    }
}

#[derive(Debug)]
pub(crate) enum ReaderEvent {
    CreateWorkspace(usize),
    DestroyWorkspace(usize),
    Workspace(usize),
    LanguageChanged(String),
}

impl Stream for Reader {
    type Item = ReaderEvent;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            let line = match ready!(this.socket.as_mut().poll_next_line(cx)) {
                Ok(Some(line)) => line,
                Ok(None) => return Ready(None),
                Err(err) => {
                    log::error!("failed to read line from Hyprland reader socket: {err:?}");
                    continue;
                }
            };

            match parse_event(&line) {
                Ok(Some(event)) => return Ready(Some(event)),
                Ok(None) => continue, // event that we are not interested in
                Err(err) => {
                    log::error!("{err:?}")
                }
            }
        }
    }
}

fn parse_event(line: &str) -> Result<Option<ReaderEvent>> {
    let (event, payload) = line.split_once(">>").with_context(|| {
        format!("malformed line from Hyprland reader socket: {line:?} (expected >> separator)")
    })?;

    let num_payload = || {
        payload
            .parse::<usize>()
            .with_context(|| format!("non-numeric payload of {event} event: {payload:?}"))
    };

    let last_substring = || {
        payload.split(",").last().with_context(|| {
            format!("expected comma separator in the payload of {event}, got {payload:?}")
        })
    };

    let event = match event {
        "createworkspace" => ReaderEvent::CreateWorkspace(num_payload()?),
        "destroyworkspace" => ReaderEvent::DestroyWorkspace(num_payload()?),
        "workspace" => ReaderEvent::Workspace(num_payload()?),
        "activelayout" => ReaderEvent::LanguageChanged(last_substring()?.to_string()),
        _ => return Ok(None),
    };

    Ok(Some(event))
}
