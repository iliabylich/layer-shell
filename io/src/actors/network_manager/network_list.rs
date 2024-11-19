use crate::{event::Network, Event};
use anyhow::Result;
use dbus::nonblock::SyncConnection;
use layer_shell_dbus::nm;
use std::sync::{mpsc::Sender, Arc};

pub(crate) async fn spawn(tx: Sender<Event>, conn: Arc<SyncConnection>) {
    loop {
        if let Err(err) = tick(&tx, conn.as_ref()).await {
            log::error!("{:?}", err);
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

async fn tick(tx: &Sender<Event>, conn: &SyncConnection) -> Result<()> {
    let networks = get_networks(conn).await?;
    tx.send(Event::NetworkList(networks))?;
    Ok(())
}

async fn get_networks(conn: &SyncConnection) -> Result<Vec<Network>> {
    let mut ifaces = vec![];

    let devices = nm::NetworkManager::get_devices(conn).await?;

    for device in devices {
        match get_device(conn, &device).await {
            Ok(network) => ifaces.push(network),
            Err(_) => log::warn!("Failed to get data for Device {device:?} (not connected?)"),
        }
    }

    Ok(ifaces)
}

async fn get_device(conn: &SyncConnection, device: &nm::Device) -> Result<Network> {
    let iface = device.interface(conn).await?;
    let ip4_config = device.ip4_config(conn).await?;
    let address = ip4_config.address(conn).await?;

    Ok(Network { iface, address })
}
