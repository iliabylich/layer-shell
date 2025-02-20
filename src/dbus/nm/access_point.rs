use crate::dbus::generated::nm_access_point::OrgFreedesktopNetworkManagerAccessPoint;
use anyhow::{Context as _, Result};
use dbus::{
    Path,
    blocking::{Connection, Proxy},
};
use std::time::Duration;

pub(crate) struct AccessPoint {
    pub(crate) path: Path<'static>,
}

impl AccessPoint {
    fn proxy<'a>(&'a self, conn: &'a Connection) -> Proxy<'a, &'a Connection> {
        Proxy::new(
            "org.freedesktop.NetworkManager",
            &self.path,
            Duration::from_millis(5000),
            conn,
        )
    }

    pub(crate) fn ssid(&self, conn: &Connection) -> Result<String> {
        let ssid = self.proxy(conn).ssid().context("failed to get Ssid")?;

        let ssid = String::from_utf8(ssid).context("non UTF-8 ssid")?;

        Ok(ssid)
    }

    pub(crate) fn strength(&self, conn: &Connection) -> Result<u8> {
        self.proxy(conn)
            .strength()
            .context("failed to get Strength property")
    }
}
