use crate::env::{hyprland_instance_signature, xdg_runtime_dir};
use anyhow::{Context as _, Result, bail};
use tokio::{
    io::{AsyncBufReadExt, BufReader, Lines},
    net::UnixStream,
};

pub(crate) struct Reader {
    socket: Lines<BufReader<UnixStream>>,
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

    pub(crate) async fn next_event(&mut self) -> Result<ReaderEvent> {
        loop {
            let Some(line) = self.socket.next_line().await? else {
                bail!("Hyprland reader socket is closed, exiting");
            };

            match parse_event(&line) {
                Ok(Some(event)) => return Ok(event),
                Ok(None) => continue, // event that we are not interested in
                Err(err) => {
                    log::error!(target: "Hyprland", "{err:?}")
                }
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum ReaderEvent {
    CreateWorkspace(usize),
    DestroyWorkspace(usize),
    Workspace(usize),
    LanguageChanged(String),
}

fn parse_event(line: &str) -> Result<Option<ReaderEvent>> {
    let Some((event, payload)) = line.split_once(">>") else {
        bail!("malformed line from Hyprland reader socket: {line:?} (expected >> separator)")
    };

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
