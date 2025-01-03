use crate::dbus::gen::nm_access_point::OrgFreedesktopNetworkManagerAccessPoint;
use anyhow::{Context, Result};
use dbus::{
    nonblock::{Proxy, SyncConnection},
    Path,
};
use std::time::Duration;

pub struct AccessPoint {
    pub(crate) path: Path<'static>,
}

impl AccessPoint {
    pub async fn ssid(&self, conn: &SyncConnection) -> Result<String> {
        let ssid = Proxy::new(
            "org.freedesktop.NetworkManager",
            &self.path,
            Duration::from_millis(5000),
            conn,
        )
        .ssid()
        .await
        .context("failed to get Ssid")?;

        let ssid = String::from_utf8(ssid).context("non UTF-8 ssid")?;

        Ok(ssid)
    }

    pub async fn strength(&self, conn: &SyncConnection) -> Result<u8> {
        Proxy::new(
            "org.freedesktop.NetworkManager",
            &self.path,
            Duration::from_millis(5000),
            conn,
        )
        .strength()
        .await
        .context("failed to get Strength property")
    }
}
