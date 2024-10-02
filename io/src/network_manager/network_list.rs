use crate::Event;
use anyhow::{Context, Result};
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;

pub(crate) async fn spawn(tx: Sender<Event>) {
    if let Err(err) = try_spawn(tx).await {
        log::error!("NM model error: {}\n{}", err, err.backtrace());
    }
}

async fn try_spawn(tx: Sender<Event>) -> Result<()> {
    let dbus = zbus::Connection::system()
        .await
        .context("failed to connect to system DBus")?;

    loop {
        match get_state(&dbus).await {
            Ok(ifaces) => {
                if tx.send(Event::NetworkList(ifaces)).await.is_err() {
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

async fn get_state(dbus: &zbus::Connection) -> Result<Vec<(String, String)>> {
    let mut ifaces = vec![];

    let device_ids = get_device_ids(dbus).await?;

    for device_id in device_ids {
        if let Ok((name, mut ip)) = get_iface(dbus, device_id).await {
            if ip.is_none() {
                ip = Some(get_ip4_config(dbus, device_id).await?);
            }

            let ip = ip.unwrap_or_else(|| String::from("unknown"));

            ifaces.push((name, ip));
        } else {
            log::warn!("Failed to get data for Device {device_id} (not connected?)");
        }
    }

    Ok(ifaces)
}

async fn get_device_ids(dbus: &zbus::Connection) -> Result<Vec<usize>> {
    let body = dbus
        .call_method(
            Some("org.freedesktop.NetworkManager"),
            "/org/freedesktop/NetworkManager",
            Some("org.freedesktop.NetworkManager"),
            "GetAllDevices",
            &(),
        )
        .await
        .context("failed to call GetAllDevices on NetworkManager")?
        .body();

    let devices = body
        .deserialize::<Vec<zbus::zvariant::ObjectPath>>()
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

async fn get_iface(dbus: &zbus::Connection, device_id: usize) -> Result<(String, Option<String>)> {
    let body = dbus
        .call_method(
            Some("org.freedesktop.NetworkManager"),
            format!("/org/freedesktop/NetworkManager/Devices/{device_id}").as_str(),
            Some("org.freedesktop.NetworkManager.Device"),
            "GetAppliedConnection",
            &(0_u32),
        )
        .await
        .context("failed to call GetAppliedConnection on Device")?
        .body();

    let (data, _) = body
        .deserialize::<(HashMap<String, HashMap<String, zbus::zvariant::Value>>, u64)>()
        .context("failed to deserialize")?;

    let iface = data
        .get("connection")
        .context("failed to get connection field")?
        .get("interface-name")
        .context("failed to get interface-name field")?
        .downcast_ref::<zbus::zvariant::Str>()
        .context("expected interface-name to be a string")?
        .to_string();

    let address_data = data
        .get("ipv4")
        .context("failed to get ipv4 field")?
        .get("address-data")
        .context("failed to get address-data field")?
        .downcast_ref::<zbus::zvariant::Array>()
        .context("expected address-data to be an array")?;
    let ip = ip_from_address_data(address_data).ok();

    Ok((iface, ip))
}

fn ip_from_address_data(variant: zbus::zvariant::Array<'_>) -> Result<String> {
    let map = variant
        .first()
        .context("expected at least one element")?
        .downcast_ref::<zbus::zvariant::Dict>()
        .context("expected 1st element to be Dict")?;
    let address: zbus::zvariant::Str = map
        .get(&"address")
        .context("expected address field to be a Str")?
        .context("no address field")?;
    Ok(address.to_string())
}

async fn get_ip4_config(dbus: &zbus::Connection, device_id: usize) -> Result<String> {
    let body = dbus
        .call_method(
            Some("org.freedesktop.NetworkManager"),
            format!("/org/freedesktop/NetworkManager/IP4Config/{device_id}").as_str(),
            Some("org.freedesktop.DBus.Properties"),
            "Get",
            &("org.freedesktop.NetworkManager.IP4Config", "AddressData"),
        )
        .await
        .context("failed to call Get on Device")?
        .body();

    let res = body
        .deserialize::<zbus::zvariant::Value>()
        .context("expected Variant")?;

    ip_from_address_data(
        res.downcast_ref::<zbus::zvariant::Array>()
            .context("expected an array")?,
    )
}
