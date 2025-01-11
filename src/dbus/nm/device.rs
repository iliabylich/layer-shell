use crate::dbus::{
    gen::{
        nm_device::OrgFreedesktopNetworkManagerDevice,
        nm_device_wireless::OrgFreedesktopNetworkManagerDeviceWireless,
    },
    nm::{AccessPoint, Ip4Config},
};
use anyhow::{Context, Result};
use dbus::{
    blocking::{Connection, Proxy},
    Path,
};
use std::time::Duration;

#[derive(Debug)]
pub struct Device {
    pub(crate) path: Path<'static>,
}

impl Device {
    pub fn interface(&self, conn: &Connection) -> Result<String> {
        Proxy::new(
            "org.freedesktop.NetworkManager",
            &self.path,
            Duration::from_millis(5000),
            conn,
        )
        .interface()
        .context("failed to get Interface property on Device")
    }

    pub fn ip4_config(&self, conn: &Connection) -> Result<Ip4Config> {
        let path = Proxy::new(
            "org.freedesktop.NetworkManager",
            &self.path,
            Duration::from_millis(5000),
            conn,
        )
        .ip4_config()
        .context("failed to get IP4Config property on Device")?;

        Ok(Ip4Config { path })
    }

    pub fn active_access_point(&self, conn: &Connection) -> Result<AccessPoint> {
        let path = Proxy::new(
            "org.freedesktop.NetworkManager",
            &self.path,
            Duration::from_millis(5000),
            conn,
        )
        .active_access_point()
        .context("failed to get ActiveAccessPoint on Device")?;

        Ok(AccessPoint { path })
    }
}
