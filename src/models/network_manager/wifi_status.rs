use gtk4::{
    gio::{DBusCallFlags, DBusConnection},
    glib::{
        prelude::{FromVariant, ToVariant},
        Variant,
    },
};
use std::collections::HashMap;

#[derive(Debug)]
enum WiFiStatusError {
    NoWlo1Interface,
    NoActiveAccessPoint,
    NoAccessPointData,
}

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
                        eprintln!("WiFiStatus error: {err:?}")
                    }
                }

                gtk4::glib::timeout_future_seconds(1).await;
            }
        });
    }

    async fn get_state(iface: &str) -> Result<WiFiStatus, WiFiStatusError> {
        let dbus = new_dbus().await;
        let device_path = get_device_path_by_iface(&dbus, iface)
            .await
            .ok_or(WiFiStatusError::NoWlo1Interface)?;
        let access_point_path = get_access_point_path(&dbus, &device_path)
            .await
            .ok_or(WiFiStatusError::NoActiveAccessPoint)?;
        let (ssid, strength) = get_ssid_and_strength(&dbus, &access_point_path)
            .await
            .ok_or(WiFiStatusError::NoAccessPointData)?;
        Ok(WiFiStatus { ssid, strength })
    }
}

async fn new_dbus() -> DBusConnection {
    gtk4::gio::bus_get_future(gtk4::gio::BusType::System)
        .await
        .unwrap()
}

async fn get_device_path_by_iface(dbus: &DBusConnection, iface: &str) -> Option<String> {
    use gtk4::glib;
    #[derive(glib::Variant)]
    struct Request {
        iface: String,
    }
    let req = Request {
        iface: iface.to_string(),
    }
    .to_variant();

    dbus.call_future(
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
    .unwrap()
    .child_value(0)
    .str()?
    .to_string()
    .into()
}

async fn get_access_point_path(dbus: &DBusConnection, device_path: &str) -> Option<String> {
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

    let variant = dbus
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
        .unwrap()
        .child_value(0)
        .child_value(0);

    variant.str().map(|s| s.to_string())
}

async fn get_ssid_and_strength(
    dbus: &DBusConnection,
    access_point_path: &str,
) -> Option<(String, u8)> {
    use gtk4::glib;
    #[derive(glib::Variant)]
    struct Request {
        dbus_interface: String,
    }
    let req = Request {
        dbus_interface: String::from("org.freedesktop.NetworkManager.AccessPoint"),
    }
    .to_variant();

    let variant = dbus
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
        .unwrap()
        .child_value(0);

    let props = HashMap::<String, Variant>::from_variant(&variant)?;

    let ssid = props.get("Ssid")?.data();
    let ssid = String::from_utf8(ssid.to_vec()).ok()?;

    let strength = props.get("Strength")?.get::<u8>()?;

    Some((ssid, strength))
}
