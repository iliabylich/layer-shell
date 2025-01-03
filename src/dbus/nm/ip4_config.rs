use crate::dbus::gen::nm_ip4_config::OrgFreedesktopNetworkManagerIP4Config;
use anyhow::{Context, Result};
use dbus::{
    arg::{RefArg, Variant},
    nonblock::{Proxy, SyncConnection},
    Path,
};
use std::{collections::HashMap, time::Duration};

pub struct Ip4Config {
    pub(crate) path: Path<'static>,
}

impl Ip4Config {
    async fn address_data(
        &self,
        conn: &SyncConnection,
    ) -> Result<Vec<HashMap<String, Variant<Box<dyn RefArg>>>>> {
        Proxy::new(
            "org.freedesktop.NetworkManager",
            &self.path,
            Duration::from_millis(5000),
            conn,
        )
        .address_data()
        .await
        .context("failed to get AddressData property on Ip4Config")
    }

    pub async fn address(&self, conn: &SyncConnection) -> Result<String> {
        let address_data = self.address_data(conn).await?;
        let address_data = address_data.first().context("expected at least 1 item")?;
        let address = address_data.get("address").context("no address key")?;
        let address = address.as_str().context("address is not a string")?;
        Ok(address.to_string())
    }
}
