use crate::{global, Event};
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

pub(crate) async fn on_command(cmd: layer_shell_pipewire::Command) {
    if let Err(err) = PW_RX::get().send(cmd) {
        log::error!("Failed to send command to PW: {:?}", err);
    }
}

impl From<layer_shell_pipewire::Event> for Event {
    fn from(e: layer_shell_pipewire::Event) -> Self {
        match e {
            layer_shell_pipewire::Event::Volume(e) => Self::Volume(e),
        }
    }
}
