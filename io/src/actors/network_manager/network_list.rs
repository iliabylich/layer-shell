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
    if let Err(err) = try_spawn(tx, conn).await {
        log::error!("NM model error: {}\n{}", err, err.backtrace());
    }
}

async fn try_spawn(tx: Sender<Event>, conn: Arc<SyncConnection>) -> Result<()> {
    loop {
        match get_networks(Arc::clone(&conn)).await {
            Ok(ifaces) => {
                if tx.send(Event::NetworkList(ifaces)).is_err() {
                    log::error!("failed to send NetworkList event");
                }
            }
            Err(err) => {
                log::error!("NetworkList error: {}\n{}", err, err.backtrace());
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

async fn get_networks(conn: Arc<SyncConnection>) -> Result<Vec<Network>> {
    let mut ifaces = vec![];

    let devices = get_devices(Arc::clone(&conn)).await?;

    for device in devices {
        if let Ok(network) = get_device(Arc::clone(&conn), &device).await {
            ifaces.push(network);
        } else {
            log::warn!("Failed to get data for Device {device} (not connected?)");
        }
    }

    Ok(ifaces)
}

async fn get_devices(conn: Arc<SyncConnection>) -> Result<Vec<Path<'static>>> {
    Proxy::new(
        "org.freedesktop.NetworkManager",
        "/org/freedesktop/NetworkManager",
        Duration::from_millis(5000),
        Arc::clone(&conn),
    )
    .get_devices()
    .await
    .context("failed to get devices")
}

async fn get_device(conn: Arc<SyncConnection>, device: &Path<'static>) -> Result<Network> {
    let device_proxy = Proxy::new(
        "org.freedesktop.NetworkManager",
        device,
        Duration::from_millis(5000),
        Arc::clone(&conn),
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
        Arc::clone(&conn),
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
