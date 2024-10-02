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
        let state = get_state(&dbus, "wlo1")
            .await
            .inspect_err(|err| log::error!("WiFiStatus error: {}\n{}", err, err.backtrace()))
            .ok();

        if tx.send(Event::WiFi(state)).await.is_err() {
            log::error!("failed to send WiFi event");
        }

        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}

async fn get_state(dbus: &zbus::Connection, iface: &str) -> Result<(String, u8)> {
    let device_path = get_device_path_by_iface(dbus, iface).await?;
    let access_point_path = get_access_point_path(dbus, &device_path).await?;
    get_ssid_and_strength(dbus, &access_point_path).await
}

async fn get_device_path_by_iface(dbus: &zbus::Connection, iface: &str) -> Result<String> {
    let body = dbus
        .call_method(
            Some("org.freedesktop.NetworkManager"),
            "/org/freedesktop/NetworkManager",
            Some("org.freedesktop.NetworkManager"),
            "GetDeviceByIpIface",
            &iface,
        )
        .await
        .context("failed to call GetDeviceByIface on NetworkManager")?
        .body();

    let res = body
        .deserialize::<zbus::zvariant::ObjectPath>()
        .context("failed to deserialize response")?;

    Ok(res.as_str().to_string())
}

async fn get_access_point_path(dbus: &zbus::Connection, device_path: &str) -> Result<String> {
    let body = dbus
        .call_method(
            Some("org.freedesktop.NetworkManager"),
            device_path,
            Some("org.freedesktop.DBus.Properties"),
            "Get",
            &(
                "org.freedesktop.NetworkManager.Device.Wireless",
                "ActiveAccessPoint",
            ),
        )
        .await
        .context("failed to call Get on WiFi Device")?
        .body();

    let variant = body
        .deserialize::<zbus::zvariant::Value>()
        .context("failed to deserialize output of active access point")?;

    let path = variant
        .downcast_ref::<zbus::zvariant::ObjectPath>()
        .context("expected output to be ObjectPath")?;

    Ok(path.as_str().to_string())
}

async fn get_ssid_and_strength(
    dbus: &zbus::Connection,
    access_point_path: &str,
) -> Result<(String, u8)> {
    let body = dbus
        .call_method(
            Some("org.freedesktop.NetworkManager"),
            access_point_path,
            Some("org.freedesktop.DBus.Properties"),
            "GetAll",
            &("org.freedesktop.NetworkManager.AccessPoint"),
        )
        .await
        .context("failed to call GetAll on access point")?
        .body();

    let properties = body
        .deserialize::<HashMap<String, zbus::zvariant::Value>>()
        .context("failed to parse response of GetAll on access point")?;

    let ssid = properties
        .get("Ssid")
        .context("failed to get Ssid property")?
        .downcast_ref::<zbus::zvariant::Array>()
        .context("expected Ssid to be an array")?;
    let ssid = Vec::<u8>::try_from(ssid).context("expected Ssid to be an array of bytes")?;
    let ssid = String::from_utf8(ssid).context("non-utf-8 Ssid")?;

    let strength = properties
        .get("Strength")
        .context("failed to get Strength property")?
        .downcast_ref::<u8>()
        .context("Strength property is not a number")?;

    Ok((ssid, strength))
}
