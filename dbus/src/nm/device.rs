use crate::{
    gen::{
        nm_device::OrgFreedesktopNetworkManagerDevice,
        nm_device_wireless::OrgFreedesktopNetworkManagerDeviceWireless,
    },
    nm::{AccessPoint, Ip4Config},
};
use anyhow::{Context, Result};
use dbus::{
    nonblock::{Proxy, SyncConnection},
    Path,
};
use std::time::Duration;

#[derive(Debug)]
pub struct Device {
    pub(crate) path: Path<'static>,
}

impl Device {
    pub async fn interface(&self, conn: &SyncConnection) -> Result<String> {
        Proxy::new(
            "org.freedesktop.NetworkManager",
            &self.path,
            Duration::from_millis(5000),
            conn,
        )
        .interface()
        .await
        .context("failed to get Interface property on Device")
    }

    pub async fn ip4_config(&self, conn: &SyncConnection) -> Result<Ip4Config> {
        let path = Proxy::new(
            "org.freedesktop.NetworkManager",
            &self.path,
            Duration::from_millis(5000),
            conn,
        )
        .ip4_config()
        .await
        .context("failed to get IP4Config property on Device")?;

        Ok(Ip4Config { path })
    }

    pub async fn active_access_point(&self, conn: &SyncConnection) -> Result<AccessPoint> {
        let path = Proxy::new(
            "org.freedesktop.NetworkManager",
            &self.path,
            Duration::from_millis(5000),
            conn,
        )
        .active_access_point()
        .await
        .context("failed to get ActiveAccessPoint on Device")?;

        Ok(AccessPoint { path })
    }
}
