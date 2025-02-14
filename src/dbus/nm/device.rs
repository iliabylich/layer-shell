use crate::dbus::{
    gen::{
        nm_device::OrgFreedesktopNetworkManagerDevice,
        nm_device_statistics::OrgFreedesktopNetworkManagerDeviceStatistics,
        nm_device_wireless::OrgFreedesktopNetworkManagerDeviceWireless,
    },
    nm::{AccessPoint, Ip4Config},
};
use anyhow::{Context as _, Result};
use dbus::{
    blocking::{Connection, Proxy},
    Path,
};
use std::time::Duration;

#[derive(Debug)]
pub(crate) struct Device {
    pub(crate) path: Path<'static>,
}

impl Device {
    fn proxy<'a>(&'a self, conn: &'a Connection) -> Proxy<'a, &'a Connection> {
        Proxy::new(
            "org.freedesktop.NetworkManager",
            &self.path,
            Duration::from_millis(5000),
            conn,
        )
    }

    pub(crate) fn interface(&self, conn: &Connection) -> Result<String> {
        self.proxy(conn)
            .interface()
            .context("failed to get Interface property on Device")
    }

    pub(crate) fn ip4_config(&self, conn: &Connection) -> Result<Ip4Config> {
        let path = self
            .proxy(conn)
            .ip4_config()
            .context("failed to get IP4Config property on Device")?;

        Ok(Ip4Config { path })
    }

    pub(crate) fn active_access_point(&self, conn: &Connection) -> Result<AccessPoint> {
        let path = self
            .proxy(conn)
            .active_access_point()
            .context("failed to get ActiveAccessPoint on Device")?;

        Ok(AccessPoint { path })
    }

    pub(crate) fn set_refresh_rate_in_ms(
        &self,
        conn: &Connection,
        refresh_rate_in_ms: u32,
    ) -> Result<()> {
        self.proxy(conn)
            .set_refresh_rate_ms(refresh_rate_in_ms)
            .context("failed to update RefreshRateMs")
    }
}
