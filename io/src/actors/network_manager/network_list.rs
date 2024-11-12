use crate::{
    dbus::{
        nm::OrgFreedesktopNetworkManager as _, nm_device::OrgFreedesktopNetworkManagerDevice as _,
        nm_ip4_config::OrgFreedesktopNetworkManagerIP4Config as _,
    },
    event::Network,
    Event,
};
use anyhow::{Context, Result};
use dbus::{
    arg::RefArg,
    nonblock::{Proxy, SyncConnection},
    Path,
};
use std::{
    sync::{mpsc::Sender, Arc},
    time::Duration,
};

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

    let devices = get_devices(conn).await?;

    for device in devices {
        match get_device(conn, &device).await {
            Ok(network) => ifaces.push(network),
            Err(_) => log::warn!("Failed to get data for Device {device} (not connected?)"),
        }
    }

    Ok(ifaces)
}

async fn get_devices(conn: &SyncConnection) -> Result<Vec<Path<'static>>> {
    Proxy::new(
        "org.freedesktop.NetworkManager",
        "/org/freedesktop/NetworkManager",
        Duration::from_millis(5000),
        conn,
    )
    .get_devices()
    .await
    .context("failed to get devices")
}

async fn get_device(conn: &SyncConnection, device: &Path<'static>) -> Result<Network> {
    let device_proxy = Proxy::new(
        "org.freedesktop.NetworkManager",
        device,
        Duration::from_millis(5000),
        conn,
    );

    let iface = device_proxy
        .interface()
        .await
        .context("failed to get Interface property on Device")?;

    let ip4_config = device_proxy
        .ip4_config()
        .await
        .context("failed to get IP4Config property on Device")?;

    let ip4_config_proxy = Proxy::new(
        "org.freedesktop.NetworkManager",
        ip4_config,
        Duration::from_millis(5000),
        conn,
    );
    let data = ip4_config_proxy
        .address_data()
        .await
        .context("failed to get AddressData property on Ip4Config")?;

    let address_data = data.first().context("expected at least 1 item")?;
    let address = address_data.get("address").context("no address key")?;
    let address = address.as_str().context("address is not a string")?;

    Ok(Network {
        iface,
        ip: address.to_string(),
    })
}
