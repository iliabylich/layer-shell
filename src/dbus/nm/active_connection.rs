use crate::dbus::{
    generated::nm_active_connection::OrgFreedesktopNetworkManagerConnectionActive as _, nm::Device,
};
use anyhow::{Context as _, Result};
use dbus::{
    Path,
    blocking::{Connection, Proxy},
};
use std::time::Duration;

pub(crate) struct ActiveConnection {
    pub(crate) path: Path<'static>,
}

impl ActiveConnection {
    fn proxy<'a>(&'a self, conn: &'a Connection) -> Proxy<'a, &'a Connection> {
        Proxy::new(
            "org.freedesktop.NetworkManager",
            &self.path,
            Duration::from_millis(5000),
            conn,
        )
    }

    pub(crate) fn type_(&self, conn: &Connection) -> Result<String> {
        self.proxy(conn)
            .type_()
            .context("failed to get Type property")
    }

    pub(crate) fn devices(&self, conn: &Connection) -> Result<Vec<Device>> {
        let paths = self
            .proxy(conn)
            .devices()
            .context("failed to get Devices property")?;

        Ok(paths.into_iter().map(|path| Device { path }).collect())
    }
}
