use crate::{
    dbus::{
        nm::OrgFreedesktopNetworkManager as _,
        nm_access_point::OrgFreedesktopNetworkManagerAccessPoint as _,
        nm_device_wireless::OrgFreedesktopNetworkManagerDeviceWireless as _,
    },
    event::WiFiStatus,
    Event,
};
use anyhow::{Context, Result};
use dbus::nonblock::{Proxy, SyncConnection};
use std::{
    sync::{mpsc::Sender, Arc},
    time::Duration,
};

pub(crate) async fn spawn(tx: Sender<Event>, conn: Arc<SyncConnection>) {
    loop {
        if let Err(err) = tick(&tx, conn.as_ref()).await {
            log::error!("{:?}", err);
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}

async fn tick(tx: &Sender<Event>, conn: &SyncConnection) -> Result<()> {
    let state = get_status(conn, "wlo1")
        .await
        .inspect_err(|err| log::error!("WiFiStatus error: {:?}", err))
        .ok();

    tx.send(Event::WiFiStatus(state))?;

    Ok(())
}

async fn get_status(conn: &SyncConnection, iface: &str) -> Result<WiFiStatus> {
    let device = Proxy::new(
        "org.freedesktop.NetworkManager",
        "/org/freedesktop/NetworkManager",
        Duration::from_millis(5000),
        conn,
    )
    .get_device_by_ip_iface(iface)
    .await
    .context("failed to call GetDeviceByIface on NetworkManager")?;

    let access_point = Proxy::new(
        "org.freedesktop.NetworkManager",
        device,
        Duration::from_millis(5000),
        conn,
    )
    .active_access_point()
    .await
    .context("failed to get ActiveAccessPoint on Device")?;

    let access_point_proxy = Proxy::new(
        "org.freedesktop.NetworkManager",
        access_point,
        Duration::from_millis(5000),
        conn,
    );

    let ssid = access_point_proxy
        .ssid()
        .await
        .context("failed to get Ssid")?;
    let ssid = String::from_utf8(ssid).context("non UTF-8 ssid")?;

    let strength = access_point_proxy
        .strength()
        .await
        .context("failed to get Strength property")?;

    Ok(WiFiStatus { ssid, strength })
}
