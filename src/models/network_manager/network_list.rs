use crate::utils::singleton;
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
                match Self::get_state().await {
                    Ok(ifaces) => {
                        Self::get().list = ifaces;
                    }
                    Err(err) => {
                        eprintln!("failed to get list of networks:\n{}", err);
                    }
                }

                gtk4::glib::timeout_future_seconds(5).await;
            }
        });
    }

    pub(crate) fn get_current() -> &'static [Iface] {
        &Self::get().list
    }

    async fn get_state() -> Result<Vec<Iface>> {
        let mut ifaces = vec![];

        let dbus = new_dbus().await?;
        let device_ids = get_device_ids(&dbus).await?;

        for device_id in device_ids {
            if let Ok((name, mut ip)) = get_iface(&dbus, device_id).await {
                if ip.is_none() {
                    ip = Some(get_ip4_config(&dbus, device_id).await?);
                }

                let ip = ip.unwrap_or_else(|| String::from("unknown"));

                ifaces.push(Iface { name, ip });
            } else {
                println!("Failed to get data for Device {device_id} (not connected?)");
            }
        }

        Ok(ifaces)
    }
}

async fn new_dbus() -> Result<DBusConnection> {
    gtk4::gio::bus_get_future(gtk4::gio::BusType::System)
        .await
        .context("failed to connect to DBus")
}

async fn get_device_ids(dbus: &DBusConnection) -> Result<Vec<usize>> {
    use gtk4::glib;
    #[derive(Debug, glib::Variant)]
    struct Response {
        devices: Vec<String>,
    }
    let res = dbus
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
        .context("failed to call GetAllDevices on NetworkManager")?;

    let devices = Response::from_variant(&res)
        .context("unexpected GetAllDevices response from NetworkManager")?
        .devices;

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

async fn get_iface(dbus: &DBusConnection, device_id: usize) -> Result<(String, Option<String>)> {
    use gtk4::glib;
    #[derive(Debug, glib::Variant)]
    struct Request(u32);
    let req = Request(0).to_variant();

    #[derive(Debug, glib::Variant)]
    struct Response {
        data: HashMap<String, HashMap<String, Variant>>,
        n: u64,
    }

    let res = dbus
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
        .context("failed to call GetAppliedConnection on Device")?;

    let data = Response::from_variant(&res).unwrap().data;
    let iface = data
        .get("connection")
        .context("failed to get connection field")?
        .get("interface-name")
        .context("failed to get interface-name field")?
        .str()
        .context("expected interface-name to be a string")?
        .to_string();

    let address_data = data
        .get("ipv4")
        .context("failed to get ipv4 field")?
        .get("address-data")
        .context("failed to get address-data field")?;
    let ip = ip_from_address_data(address_data).ok();

    Ok((iface, ip))
}

fn ip_from_address_data(variant: &Variant) -> Result<String> {
    let addresses =
        Vec::<HashMap<String, Variant>>::from_variant(variant).context("not a Vec<HashMap>")?;
    let first_address = addresses
        .first()
        .context("expected at least one ip address")?;
    let address = first_address.get("address").context("no address field")?;
    address
        .str()
        .map(|s| s.to_string())
        .context("expected address to be a string")
}

async fn get_ip4_config(dbus: &DBusConnection, device_id: usize) -> Result<String> {
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

    let res = dbus
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
        .context("failed to call Get on Device")?;

    ip_from_address_data(&res.child_value(0).child_value(0))
}
