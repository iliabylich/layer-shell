use crate::{event::Network, Event};
use anyhow::{Context, Result};
use dbus::{
    arg::{RefArg, Variant},
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

    let device_ids = get_device_ids(Arc::clone(&conn)).await?;

    for device_id in device_ids {
        if let Ok((iface, mut ip)) = get_iface(Arc::clone(&conn), device_id).await {
            if ip.is_none() {
                ip = Some(get_ip4_config(Arc::clone(&conn), device_id).await?);
            }

            let ip = ip.unwrap_or_else(|| String::from("unknown"));

            ifaces.push(Network { iface, ip });
        } else {
            log::warn!("Failed to get data for Device {device_id} (not connected?)");
        }
    }

    Ok(ifaces)
}

async fn get_device_ids(conn: Arc<SyncConnection>) -> Result<Vec<usize>> {
    let (devices,): (Vec<Path>,) = Proxy::new(
        "org.freedesktop.NetworkManager",
        "/org/freedesktop/NetworkManager",
        Duration::from_millis(5000),
        conn,
    )
    .method_call("org.freedesktop.NetworkManager", "GetAllDevices", ())
    .await
    .context("unexpected GetAllDevices response from NetworkManager")?;

    let mut out = vec![];

    for device in devices {
        let device_id = device
            .split('/')
            .last()
            .context("wrong DBus path format")?
            .parse::<usize>()
            .context("expected DBus path to end with a numner")?;
        out.push(device_id);
    }

    Ok(out)
}

async fn get_iface(
    conn: Arc<SyncConnection>,
    device_id: usize,
) -> Result<(String, Option<String>)> {
    let (data, _): (
        HashMap<String, HashMap<String, Variant<Box<dyn RefArg>>>>,
        u64,
    ) = Proxy::new(
        "org.freedesktop.NetworkManager",
        format!("/org/freedesktop/NetworkManager/Devices/{device_id}"),
        Duration::from_millis(5000),
        conn,
    )
    .method_call(
        "org.freedesktop.NetworkManager.Device",
        "GetAppliedConnection",
        (0_u32,),
    )
    .await
    .context("failed to call GetAppliedConnection on Device")?;

    let iface = data
        .get("connection")
        .context("failed to get connection field")?
        .get("interface-name")
        .context("failed to get interface-name field")?
        .as_str()
        .context("expected interface-name to be a string")?
        .to_string();

    let address_data = data
        .get("ipv4")
        .context("failed to get ipv4 field")?
        .get("address-data")
        .context("failed to get address-data field")?
        .as_iter()
        .context("expected address-data to be an array")?
        .collect::<Vec<_>>();
    let ip = ip_from_address_data(address_data).ok();

    Ok((iface, ip))
}

fn ip_from_address_data(variants: Vec<&dyn RefArg>) -> Result<String> {
    let variant = variants
        .into_iter()
        .next()
        .context("expected at least 1 element")?
        .box_clone();
    let map = variant
        .as_static_inner(0)
        .context("expected at least 1 element")?;

    let mut iter = map.as_iter().context("not iterable")?;
    while let Some(key) = iter.next() {
        let value = iter.next().context("odd number of hash items")?;
        if key.as_str() == Some("address") {
            return Ok(value
                .as_str()
                .context("expected address to be a string")?
                .to_string());
        }
    }

    anyhow::bail!("no address field")
}

async fn get_ip4_config(conn: Arc<SyncConnection>, device_id: usize) -> Result<String> {
    let (data,): (Variant<Box<dyn RefArg>>,) = Proxy::new(
        "org.freedesktop.NetworkManager",
        format!("/org/freedesktop/NetworkManager/IP4Config/{device_id}"),
        Duration::from_millis(5000),
        conn,
    )
    .method_call(
        "org.freedesktop.DBus.Properties",
        "Get",
        ("org.freedesktop.NetworkManager.IP4Config", "AddressData"),
    )
    .await
    .context("failed to call Get on Device")?;

    let variant = data
        .as_iter()
        .context("expected iterable")?
        .collect::<Vec<_>>();

    ip_from_address_data(variant)
}
