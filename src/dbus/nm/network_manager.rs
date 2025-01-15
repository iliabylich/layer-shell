use crate::dbus::{
    gen::nm::OrgFreedesktopNetworkManager,
    nm::{ActiveConnection, Device},
};
use anyhow::{Context, Result};
use dbus::blocking::{Connection, Proxy};
use std::time::Duration;

pub(crate) struct NetworkManager;

impl NetworkManager {
    fn proxy(conn: &Connection) -> Proxy<'_, &'_ Connection> {
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
}
