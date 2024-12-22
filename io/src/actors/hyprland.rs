use crate::Event;
use futures::{pin_mut, StreamExt};
use std::sync::mpsc::Sender;

pub(crate) async fn spawn(tx: Sender<Event>) {
    let hyprland_stream = layer_shell_hyprland::connect()
        .await
        .map(|event| match event {
            layer_shell_hyprland::Event::Workspaces(workspaces) => Event::Workspaces(workspaces),
            layer_shell_hyprland::Event::Language(lang) => Event::Language(lang),
        });
    pin_mut!(hyprland_stream);

    while let Some(event) = hyprland_stream.next().await {
        if let Err(err) = tx.send(event) {
            log::error!("Failed to send event: {:?}", err);
        }
    }
}
