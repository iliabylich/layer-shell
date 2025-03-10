use crate::{
    Event,
    dbus::nm::{Device, NetworkManager},
    event::Network as NetworkData,
};
use anyhow::Result;
use dbus::blocking::Connection;

pub(crate) fn load(conn: &Connection) -> Event {
    Event::NetworkList {
        list: get_network_list(conn).unwrap_or_default().into(),
    }
}

fn get_network_list(conn: &Connection) -> Result<Vec<NetworkData>> {
    let mut ifaces = vec![];

    let devices = NetworkManager::get_devices(conn)?;

    for device in devices {
        match get_network_for_device(&device, conn) {
            Ok(network) => ifaces.push(network),
            Err(_) => log::warn!("Failed to get data for Device {device:?} (not connected?)"),
        }
    }

    Ok(ifaces)
}

fn get_network_for_device(device: &Device, conn: &Connection) -> Result<NetworkData> {
    let iface = device.interface(conn)?;
    let ip4_config = device.ip4_config(conn)?;
    let address = ip4_config.address(conn)?;

    Ok(NetworkData {
        iface: iface.into(),
        address: address.into(),
    })
}
