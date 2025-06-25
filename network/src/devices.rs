use crate::{
    NetworkEvent, NetworkListEvent, event::NetworkData, nm_event::NetworkManagerEvent,
    stream_map::StreamMap,
};
use anyhow::{Context as _, Result};
use futures::StreamExt;
use zbus::zvariant::OwnedObjectPath;

mod dbus {
    use zbus::proxy;

    #[proxy(
        interface = "org.freedesktop.NetworkManager",
        default_service = "org.freedesktop.NetworkManager",
        default_path = "/org/freedesktop/NetworkManager"
    )]
    pub(crate) trait NetworkManager {
        #[zbus(signal)]
        fn state_changed(&self, state: u32) -> zbus::Result<()>;

        #[zbus(property)]
        fn devices(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
    }

    #[proxy(
        default_service = "org.freedesktop.NetworkManager",
        interface = "org.freedesktop.NetworkManager.Device",
        assume_defaults = true
    )]
    pub(crate) trait Device {
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
    pub(crate) trait IP4Config {
        #[zbus(property)]
        fn address_data(
            &self,
        ) -> zbus::Result<Vec<std::collections::HashMap<String, zbus::zvariant::OwnedValue>>>;
    }
}

pub(crate) struct Devices;

const DEVICES_CHANGED: &str = "DEVICES_CHANGED";

impl Devices {
    pub(crate) async fn subscribe(
        conn: &zbus::Connection,
        stream_map: &mut StreamMap,
    ) -> Result<()> {
        let nm = dbus::NetworkManagerProxy::new(conn).await?;

        let devices = nm.devices().await.ok();
        let devices_sub = nm
            .receive_devices_changed()
            .await
            .filter_map(|e| async move {
                let paths = e.get().await.ok()?;
                Some(NetworkManagerEvent::Devices(paths))
            });

        stream_map.add(DEVICES_CHANGED, devices_sub);
        if let Some(paths) = devices {
            stream_map.emit(NetworkManagerEvent::Devices(paths))?;
        }
        Ok(())
    }

    pub(crate) async fn changed(
        paths: Vec<OwnedObjectPath>,
        conn: &zbus::Connection,
    ) -> NetworkEvent {
        let mut list = vec![];

        for path in paths {
            match Self::get_device_network(path, conn).await {
                Ok(network) => list.push(network),
                Err(err) => log::error!("{err:?}"),
            }
        }

        NetworkEvent::NetworkList(NetworkListEvent { list: list.into() })
    }

    async fn get_device_network(
        path: OwnedObjectPath,
        conn: &zbus::Connection,
    ) -> Result<NetworkData> {
        let device = dbus::DeviceProxy::builder(conn)
            .path(&path)?
            .build()
            .await?;
        let iface = device.interface().await?;
        let ip4_config_path = device.ip4_config().await?;
        let ip4_config = dbus::IP4ConfigProxy::builder(conn)
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

        Ok(NetworkData {
            iface: iface.into(),
            address: address.into(),
        })
    }
}
