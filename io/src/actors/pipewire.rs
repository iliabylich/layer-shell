use crate::{global, Command, Event};
use anyhow::Result;
use std::sync::mpsc::Sender;

global!(PW_RX, Sender<layer_shell_pipewire::Command>);

pub(crate) async fn spawn(tx: Sender<Event>) {
    if let Err(err) = try_spawn(tx).await {
        log::error!("{:?}", err);
    }
}

async fn try_spawn(tx: Sender<Event>) -> Result<()> {
    let (pw_rx, pw_tx) = layer_shell_pipewire::start();
    PW_RX::set(pw_rx);

    loop {
        while let Ok(pw_event) = pw_tx.try_recv() {
            let event = Event::from(pw_event);
            if let Err(err) = tx.send(event) {
                log::error!("Failed to send event: {:?}", err);
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}

pub(crate) async fn on_command(command: &Command) {
    if let Ok(pw_command) = layer_shell_pipewire::Command::try_from(command) {
        if let Err(err) = PW_RX::get().send(pw_command) {
            log::error!("Failed to send command to PW: {:?}", err);
        }
    }
}
