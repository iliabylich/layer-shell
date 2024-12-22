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
    let mut hyprland_stream = layer_shell_hyprland::connect()
        .await?
        .map(|event| match event {
            layer_shell_hyprland::Event::Workspaces(workspaces) => Event::Workspaces(workspaces),
            layer_shell_hyprland::Event::Language(lang) => Event::Language(lang),
        });

    while let Some(event) = hyprland_stream.next().await {
        if let Err(err) = tx.send(event) {
            log::error!("Failed to send event: {:?}", err);
        }
    }

    Ok(())
}
