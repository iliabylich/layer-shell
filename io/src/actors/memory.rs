use crate::Event;
use futures::{pin_mut, StreamExt as _};
use std::sync::mpsc::Sender;

pub(crate) async fn spawn(tx: Sender<Event>) {
    let memory_stream = layer_shell_memory::connect().map(|memory| Event::Memory(memory));
    pin_mut!(memory_stream);

    while let Some(event) = memory_stream.next().await {
        if let Err(err) = tx.send(event) {
            log::error!("Failed to send event: {:?}", err);
        }
    }
}
