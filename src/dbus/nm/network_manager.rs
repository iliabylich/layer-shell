use crate::dbus::{
    gen::nm::OrgFreedesktopNetworkManager,
    nm::{ActiveConnection, Device},
};
use anyhow::{bail, Context, Result};
use dbus::blocking::{Connection, Proxy};
use std::time::Duration;

pub(crate) struct NetworkManager;

impl NetworkManager {
    pub(crate) fn proxy(conn: &Connection) -> Proxy<'_, &'_ Connection> {
        Proxy::new(
            "org.freedesktop.NetworkManager",
            "/org/freedesktop/NetworkManager",
            Duration::from_millis(5000),
            conn,
        )
    }

    pub(crate) fn get_devices(conn: &Connection) -> Result<Vec<Device>> {
        let paths = Self::proxy(conn)
            .get_devices()
            .context("failed to get devices")?;

        Ok(paths
            .into_iter()
            .map(|path| Device { path })
            .collect::<Vec<_>>())
    }

    pub(crate) fn primary_connection(conn: &Connection) -> Result<ActiveConnection> {
        let path = Self::proxy(conn)
            .primary_connection()
            .context("failed to get PrimaryConnection property on NetworkManager")?;

        Ok(ActiveConnection { path })
    }

    pub(crate) fn primary_wireless_connection(conn: &Connection) -> Result<ActiveConnection> {
        let primary_connection = Self::primary_connection(conn)?;
        if !primary_connection.type_(conn)?.contains("wireless") {
            bail!("Default connection is not wireless");
        }
        Ok(primary_connection)
    }

    pub(crate) fn primary_wireless_device(conn: &Connection) -> Result<Device> {
        let devices = Self::primary_wireless_connection(conn)?.devices(conn)?;
        if devices.len() != 1 {
            bail!("NM returned multiple devices for active connection");
        }
        let device = devices.into_iter().next().unwrap();
        Ok(device)
    }
}
