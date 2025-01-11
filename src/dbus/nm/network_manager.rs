use crate::dbus::{gen::nm::OrgFreedesktopNetworkManager, nm::Device};
use anyhow::{Context, Result};
use dbus::blocking::{Connection, Proxy};
use std::time::Duration;

pub struct NetworkManager;

impl NetworkManager {
    pub fn get_devices(conn: &Connection) -> Result<Vec<Device>> {
        let paths = Proxy::new(
            "org.freedesktop.NetworkManager",
            "/org/freedesktop/NetworkManager",
            Duration::from_millis(5000),
            conn,
        )
        .get_devices()
        .context("failed to get devices")?;

        Ok(paths
            .into_iter()
            .map(|path| Device { path })
            .collect::<Vec<_>>())
    }

    pub fn get_device_by_ip_iface(conn: &Connection, iface: &str) -> Result<Device> {
        let path = Proxy::new(
            "org.freedesktop.NetworkManager",
            "/org/freedesktop/NetworkManager",
            Duration::from_millis(5000),
            conn,
        )
        .get_device_by_ip_iface(iface)
        .context("failed to call GetDeviceByIface on NetworkManager")?;

        Ok(Device { path })
    }
}
