use crate::dbus::gen::nm_ip4_config::OrgFreedesktopNetworkManagerIP4Config;
use anyhow::{Context, Result};
use dbus::{
    arg::{RefArg, Variant},
    blocking::{Connection, Proxy},
    Path,
};
use std::{collections::HashMap, time::Duration};

pub(crate) struct Ip4Config {
    pub(crate) path: Path<'static>,
}

impl Ip4Config {
    fn proxy<'a>(&'a self, conn: &'a Connection) -> Proxy<'a, &'a Connection> {
        Proxy::new(
            "org.freedesktop.NetworkManager",
            &self.path,
            Duration::from_millis(5000),
            conn,
        )
    }

    fn address_data(
        &self,
        conn: &Connection,
    ) -> Result<Vec<HashMap<String, Variant<Box<dyn RefArg>>>>> {
        self.proxy(conn)
            .address_data()
            .context("failed to get AddressData property on Ip4Config")
    }

    pub(crate) fn address(&self, conn: &Connection) -> Result<String> {
        let address_data = self.address_data(conn)?;
        let address_data = address_data.first().context("expected at least 1 item")?;
        let address = address_data.get("address").context("no address key")?;
        let address = address.as_str().context("address is not a string")?;
        Ok(address.to_string())
    }
}
