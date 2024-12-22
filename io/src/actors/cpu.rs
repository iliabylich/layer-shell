use crate::Event;
use futures::{pin_mut, StreamExt};
use std::sync::mpsc::Sender;

pub(crate) async fn spawn(tx: Sender<Event>) {
    let cpu_stream = layer_shell_cpu::connect().map(|usage_per_core| Event::Cpu(usage_per_core));
    pin_mut!(cpu_stream);

    while let Some(event) = cpu_stream.next().await {
        if let Err(err) = tx.send(event) {
            log::error!("Failed to send event: {:?}", err);
        }
    }
}
