use crate::utils::singleton;
use gtk4::{
    gio::{DBusCallFlags, DBusConnection},
    glib::{
        prelude::{FromVariant, ToVariant},
        Variant,
    },
};
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct Iface {
    pub(crate) name: String,
    pub(crate) ip: String,
}
#[derive(Debug)]
pub(crate) struct NetworkList {
    list: Vec<Iface>,
}
singleton!(NetworkList);

impl NetworkList {
    pub(crate) fn spawn() {
        Self::set(Self { list: vec![] });

        gtk4::glib::spawn_future_local(async {
            loop {
                let ifaces = Self::get_state().await;
                Self::get().list = ifaces;

                gtk4::glib::timeout_future_seconds(5).await;
            }
        });
    }

    pub(crate) fn get_current() -> &'static [Iface] {
        &Self::get().list
    }

    async fn get_state() -> Vec<Iface> {
        let mut ifaces = vec![];

        let dbus = new_dbus().await;
        let device_ids = get_device_ids(&dbus).await;

        for device_id in device_ids {
            if let Some((iface, ip)) = get_iface(&dbus, device_id).await {
                let ip = if let Some(ip) = ip {
                    Some(ip)
                } else {
                    get_ip4_config(&dbus, device_id).await
                };

                ifaces.push(Iface {
                    name: iface,
                    ip: ip.unwrap_or_else(|| String::from("unknown")),
                });
            }
        }

        ifaces
    }
}

async fn new_dbus() -> DBusConnection {
    gtk4::gio::bus_get_future(gtk4::gio::BusType::System)
        .await
        .unwrap()
}

async fn get_device_ids(dbus: &DBusConnection) -> Vec<usize> {
    use gtk4::glib;
    #[derive(Debug, glib::Variant)]
    struct Response {
        devices: Vec<String>,
    }
    let variant = dbus
        .call_future(
            Some("org.freedesktop.NetworkManager"),
            "/org/freedesktop/NetworkManager",
            "org.freedesktop.NetworkManager",
            "GetAllDevices",
            None,
            None,
            DBusCallFlags::NONE,
            -1,
        )
        .await
        .unwrap();

    Response::from_variant(&variant)
        .unwrap()
        .devices
        .into_iter()
        .map(|path| path.split('/').last().unwrap().parse().unwrap())
        .collect()
}

async fn get_iface(dbus: &DBusConnection, device_id: usize) -> Option<(String, Option<String>)> {
    use gtk4::glib;
    #[derive(Debug, glib::Variant)]
    struct Request(u32);
    let req = Request(0).to_variant();

    #[derive(Debug, glib::Variant)]
    struct Response {
        data: HashMap<String, HashMap<String, Variant>>,
        n: u64,
    }

    let variant = dbus
        .call_future(
            Some("org.freedesktop.NetworkManager"),
            &format!("/org/freedesktop/NetworkManager/Devices/{device_id}"),
            "org.freedesktop.NetworkManager.Device",
            "GetAppliedConnection",
            Some(&req),
            None,
            DBusCallFlags::NONE,
            -1,
        )
        .await
        .ok()?;

    let data = Response::from_variant(&variant).unwrap().data;
    let iface = data
        .get("connection")
        .unwrap()
        .get("interface-name")
        .unwrap()
        .str()
        .unwrap()
        .to_string();

    let address_data = data.get("ipv4").unwrap().get("address-data").unwrap();
    let ip = ip_from_address_data(address_data);

    Some((iface, ip))
}

fn ip_from_address_data(variant: &Variant) -> Option<String> {
    let addresses = Vec::<HashMap<String, Variant>>::from_variant(variant)?;
    let first_address = addresses.first()?;
    Some(first_address.get("address")?.str()?.to_string())
}

async fn get_ip4_config(dbus: &DBusConnection, device_id: usize) -> Option<String> {
    use gtk4::glib;
    #[derive(Debug, glib::Variant)]
    struct Request {
        iface: String,
        property: String,
    }
    let req = Request {
        iface: String::from("org.freedesktop.NetworkManager.IP4Config"),
        property: String::from("AddressData"),
    }
    .to_variant();

    let variant = dbus
        .call_future(
            Some("org.freedesktop.NetworkManager"),
            &format!("/org/freedesktop/NetworkManager/IP4Config/{device_id}"),
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

    ip_from_address_data(&variant)
}
