use crate::Event;
use futures::{pin_mut, StreamExt};
use std::sync::mpsc::Sender;

pub(crate) async fn spawn(tx: Sender<Event>) {
    let pipewire_stream = layer_shell_pipewire::connect().map(|event| match event {
        layer_shell_pipewire::Event::Volume(volume) => Event::Volume(volume),
    });
    pin_mut!(pipewire_stream);

    while let Some(event) = pipewire_stream.next().await {
        if let Err(err) = tx.send(event) {
            log::error!("Failed to send event: {:?}", err);
        }
    }
}
