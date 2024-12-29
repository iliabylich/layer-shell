use crate::modules::{app_list, cpu, hyprland, memory, network, pipewire, time, weather};
use crate::Event;
use futures::{pin_mut, Stream};
use std::sync::mpsc::Sender;
use tokio_stream::StreamExt as _;

pub(crate) async fn spawn_all(tx: Sender<Event>) {
    let stream = merged_stream().await;
    pin_mut!(stream);

    while let Some(event) = stream.next().await {
        if let Err(err) = tx.send(event) {
            log::error!("Failed to send event: {:?}", err);
        }
    }
}

async fn merged_stream() -> impl Stream<Item = Event> {
    futures::stream::empty()
        .merge(app_list::connect().await)
        .merge(cpu::connect())
        .merge(hyprland::connect().await)
        .merge(memory::connect())
        .merge(pipewire::connect())
        .merge(time::connect())
        .merge(weather::connect())
        .merge(network::connect())
}
