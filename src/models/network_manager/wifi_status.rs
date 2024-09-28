use anyhow::{Context, Result};
use gtk4::{
    gio::{DBusCallFlags, DBusConnection},
    glib::{
        prelude::{FromVariant, ToVariant},
        Variant,
    },
};
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct WiFiStatus {
    pub(crate) ssid: String,
    pub(crate) strength: u8,
}
impl WiFiStatus {
    pub(crate) fn spawn<F>(on_change: F)
    where
        F: Fn(Option<WiFiStatus>) + 'static,
    {
        gtk4::glib::spawn_future_local(async move {
            loop {
                match Self::get_state("wlo1").await {
                    Ok(state) => on_change(Some(state)),
                    Err(err) => {
                        on_change(None);
                        log::error!("WiFiStatus error: {}\n{}", err, err.backtrace());
                    }
                }

                gtk4::glib::timeout_future_seconds(1).await;
            }
        });
    }

    async fn get_state(iface: &str) -> Result<WiFiStatus> {
        let dbus = new_dbus().await;
        let device_path = get_device_path_by_iface(&dbus, iface).await?;
        let access_point_path = get_access_point_path(&dbus, &device_path).await?;
        let (ssid, strength) = get_ssid_and_strength(&dbus, &access_point_path).await?;
        Ok(WiFiStatus { ssid, strength })
    }
}

async fn new_dbus() -> DBusConnection {
    gtk4::gio::bus_get_future(gtk4::gio::BusType::System)
        .await
        .unwrap()
}

async fn get_device_path_by_iface(dbus: &DBusConnection, iface: &str) -> Result<String> {
    use gtk4::glib;
    #[derive(glib::Variant)]
    struct Request {
        iface: String,
    }
    let req = Request {
        iface: iface.to_string(),
    }
    .to_variant();

    let res = dbus
        .call_future(
            Some("org.freedesktop.NetworkManager"),
            "/org/freedesktop/NetworkManager",
            "org.freedesktop.NetworkManager",
            "GetDeviceByIpIface",
            Some(&req),
            None,
            DBusCallFlags::NONE,
            -1,
        )
        .await
        .context("failed to call GetDeviceByIface on NetworkManager")?;

    res.child_value(0)
        .str()
        .map(|s| s.to_string())
        .context("expected GetDeviceByIface on NetworkManager to return a string")
}

async fn get_access_point_path(dbus: &DBusConnection, device_path: &str) -> Result<String> {
    use gtk4::glib;
    #[derive(glib::Variant)]
    struct Request {
        dbus_interface: String,
        property_name: String,
    }
    let req = Request {
        dbus_interface: String::from("org.freedesktop.NetworkManager.Device.Wireless"),
        property_name: String::from("ActiveAccessPoint"),
    }
    .to_variant();

    let res = dbus
        .call_future(
            Some("org.freedesktop.NetworkManager"),
            device_path,
            "org.freedesktop.DBus.Properties",
            "Get",
            Some(&req),
            None,
            DBusCallFlags::NONE,
            -1,
        )
        .await
        .context("failed to call Get on WiFi Device")?;

    res.child_value(0)
        .child_value(0)
        .str()
        .map(|s| s.to_string())
        .context("expected Get on WiFi Device to return a string")
}

async fn get_ssid_and_strength(
    dbus: &DBusConnection,
    access_point_path: &str,
) -> Result<(String, u8)> {
    use gtk4::glib;
    #[derive(glib::Variant)]
    struct Request {
        dbus_interface: String,
    }
    let req = Request {
        dbus_interface: String::from("org.freedesktop.NetworkManager.AccessPoint"),
    }
    .to_variant();

    let res = dbus
        .call_future(
            Some("org.freedesktop.NetworkManager"),
            access_point_path,
            "org.freedesktop.DBus.Properties",
            "GetAll",
            Some(&req),
            None,
            DBusCallFlags::NONE,
            -1,
        )
        .await
        .context("failed to call GetAll on access point")?;

    let props = HashMap::<String, Variant>::from_variant(&res.child_value(0))
        .context("failed to parse response of GetAll on access point")?;

    let ssid = props
        .get("Ssid")
        .context("failed to get Ssid property")?
        .data();
    let ssid = String::from_utf8(ssid.to_vec()).context("non-utf-8 Ssid")?;

    let strength = props
        .get("Strength")
        .context("failed to get Strength property")?
        .get::<u8>()
        .context("Strength property is not a number")?;

    Ok((ssid, strength))
}
