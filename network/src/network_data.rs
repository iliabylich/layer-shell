use anyhow::{Context as _, Result};
use ffi::CString;
use zbus::{Connection, proxy, zvariant::OwnedObjectPath};

#[proxy(
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager.Device",
    assume_defaults = true
)]
trait Device {
    #[zbus(property)]
    fn interface(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn ip4_config(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}
#[proxy(
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager.IP4Config",
    assume_defaults = true
)]
trait IP4Config {
    #[zbus(property)]
    fn address_data(
        &self,
    ) -> zbus::Result<Vec<std::collections::HashMap<String, zbus::zvariant::OwnedValue>>>;
}

#[derive(Debug)]
#[repr(C)]
pub struct NetworkData {
    pub iface: CString,
    pub address: CString,
}

impl NetworkData {
    pub(crate) async fn read_many(conn: &Connection, paths: Vec<OwnedObjectPath>) -> Vec<Self> {
        let mut list = vec![];

        for path in paths {
            match Self::read(conn, path).await {
                Ok(network) => list.push(network),
                Err(err) => log::error!(target: "Network", "{err:?}"),
            }
        }

        list
    }

    async fn read(conn: &Connection, path: OwnedObjectPath) -> Result<Self> {
        let device = DeviceProxy::builder(conn).path(&path)?.build().await?;
        let iface = device.interface().await?;
        let ip4_config_path = device.ip4_config().await?;
        let ip4_config = IP4ConfigProxy::builder(conn)
            .path(ip4_config_path)?
            .build()
            .await?;
        let address_data = ip4_config.address_data().await?;
        let mut address_data = address_data
            .into_iter()
            .next()
            .context("expected at least 1 address")?;
        let address = address_data.remove("address").context("no address data")?;
        let address = String::try_from(address).context("address is not a string")?;

        Ok(Self {
            iface: iface.into(),
            address: address.into(),
        })
    }
}
