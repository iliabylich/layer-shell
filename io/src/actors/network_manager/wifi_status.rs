use crate::Event;
use anyhow::{Context, Result};
use dbus::{
    arg::{messageitem::MessageItem, RefArg, Variant},
    nonblock::{Proxy, SyncConnection},
    Path,
};
use std::{
    collections::HashMap,
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
        let state = get_state(Arc::clone(&conn), "wlo1")
            .await
            .inspect_err(|err| log::error!("WiFiStatus error: {}\n{}", err, err.backtrace()))
            .ok();

        if tx.send(Event::WiFi(state)).is_err() {
            log::error!("failed to send WiFi event");
        }

        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}

async fn get_state(conn: Arc<SyncConnection>, iface: &str) -> Result<(String, u8)> {
    let device_path = get_device_path_by_iface(Arc::clone(&conn), iface).await?;
    let access_point_path = get_access_point_path(Arc::clone(&conn), &device_path).await?;
    get_ssid_and_strength(Arc::clone(&conn), &access_point_path).await
}

async fn get_device_path_by_iface(conn: Arc<SyncConnection>, iface: &str) -> Result<String> {
    let (path,): (Path,) = Proxy::new(
        "org.freedesktop.NetworkManager",
        "/org/freedesktop/NetworkManager",
        Duration::from_millis(5000),
        conn,
    )
    .method_call(
        "org.freedesktop.NetworkManager",
        "GetDeviceByIpIface",
        (iface,),
    )
    .await
    .context("failed to call GetDeviceByIface on NetworkManager")?;

    Ok(path.as_str().context("expected a string")?.to_string())
}

async fn get_access_point_path(conn: Arc<SyncConnection>, device_path: &str) -> Result<String> {
    let (path,): (Variant<Path>,) = Proxy::new(
        "org.freedesktop.NetworkManager",
        device_path,
        Duration::from_millis(5000),
        conn,
    )
    .method_call(
        "org.freedesktop.DBus.Properties",
        "Get",
        (
            "org.freedesktop.NetworkManager.Device.Wireless",
            "ActiveAccessPoint",
        ),
    )
    .await
    .context("failed to call Get on WiFi Device")?;

    Ok(path.0.to_string())
}

async fn get_ssid_and_strength(
    conn: Arc<SyncConnection>,
    access_point_path: &str,
) -> Result<(String, u8)> {
    let (mut properties,): (HashMap<String, Variant<MessageItem>>,) = Proxy::new(
        "org.freedesktop.NetworkManager",
        access_point_path,
        Duration::from_millis(5000),
        conn,
    )
    .method_call(
        "org.freedesktop.DBus.Properties",
        "GetAll",
        ("org.freedesktop.NetworkManager.AccessPoint",),
    )
    .await
    .context("failed to call GetAll on access point")?;

    let ssid = properties
        .remove("Ssid")
        .context("failed to get Ssid property")?;
    let ssid = match ssid {
        Variant(MessageItem::Array(array)) => {
            let mut ssid = vec![];
            for item in array.into_vec().into_iter() {
                match item {
                    MessageItem::Byte(byte) => ssid.push(byte),
                    _ => anyhow::bail!("expected Ssid to be an array of bytes"),
                }
            }
            String::from_utf8(ssid).context("non-utf-8 Ssid")?
        }
        _ => anyhow::bail!("expected Ssid to be an array"),
    };

    let strength = properties
        .remove("Strength")
        .context("failed to get Strength property")?
        .0;

    let strength = match strength {
        MessageItem::Byte(s) => s,
        _ => anyhow::bail!("Strength property is not a number"),
    };

    Ok((ssid, strength))
}
