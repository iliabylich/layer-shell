use crate::Event;
use anyhow::Result;
use futures::{pin_mut, StreamExt};
use std::sync::mpsc::Sender;

pub(crate) async fn spawn(tx: Sender<Event>) {
    if let Err(err) = try_spawn(tx).await {
        log::error!("{:?}", err);
    }
}

async fn try_spawn(tx: Sender<Event>) -> Result<()> {
    let network_stream = layer_shell_network::connect().map(|event| match event {
        layer_shell_network::Event::WiFiStatus(wifi_status) => Event::WiFiStatus(wifi_status),
        layer_shell_network::Event::NetworkList(network_list) => Event::NetworkList(network_list),
    });
    pin_mut!(network_stream);

    while let Some(event) = network_stream.next().await {
        if let Err(err) = tx.send(event) {
            log::error!("Failed to send event: {:?}", err);
        }
    }

    Ok(())
}
