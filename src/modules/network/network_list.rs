use crate::{
    dbus::nm::{Device, NetworkManager},
    event::Network,
    Event,
};
use anyhow::Result;
use dbus::blocking::Connection;

pub(crate) fn reset(conn: &Connection) -> Result<()> {
    let event = Event::NetworkList {
        list: get_networks(conn)?.into(),
    };
    event.emit();
    Ok(())
}

fn get_networks(conn: &Connection) -> Result<Vec<Network>> {
    let mut ifaces = vec![];

    let devices = NetworkManager::get_devices(conn)?;

    for device in devices {
        match get_device(conn, &device) {
            Ok(network) => ifaces.push(network),
            Err(_) => log::warn!("Failed to get data for Device {device:?} (not connected?)"),
        }
    }

    Ok(ifaces)
}

fn get_device(conn: &Connection, device: &Device) -> Result<Network> {
    let iface = device.interface(conn)?;
    let ip4_config = device.ip4_config(conn)?;
    let address = ip4_config.address(conn)?;

    Ok(Network {
        iface: iface.into(),
        address: address.into(),
    })
}
