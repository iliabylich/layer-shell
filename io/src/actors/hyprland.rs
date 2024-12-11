use crate::Event;
use anyhow::Result;
use futures::StreamExt;
use std::sync::mpsc::Sender;

pub(crate) async fn spawn(tx: Sender<Event>) {
    if let Err(err) = try_spawn(tx).await {
        log::error!("{:?}", err);
    }
}

async fn try_spawn(tx: Sender<Event>) -> Result<()> {
    let mut hyprland_stream = layer_shell_hyprland::connect().await?;

    while let Some(event) = hyprland_stream.next().await {
        if let Err(err) = tx.send(Event::from(event)) {
            log::error!("Failed to send event: {:?}", err);
        }
    }

    Ok(())
}

impl From<layer_shell_hyprland::Event> for Event {
    fn from(e: layer_shell_hyprland::Event) -> Self {
        match e {
            layer_shell_hyprland::Event::Workspaces(e) => Self::Workspaces(e),
            layer_shell_hyprland::Event::Language(e) => Self::Language(e),
        }
    }
}
