use crate::{nm_event::NetworkManagerEvent, primary_device::PrimaryDevice, stream_map::StreamMap};
use anyhow::Result;
use futures::StreamExt;
use zbus::{Connection, zvariant::OwnedObjectPath};

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

        #[zbus(property)]
        fn primary_connection(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
    }

    #[proxy(
        default_service = "org.freedesktop.NetworkManager",
        interface = "org.freedesktop.NetworkManager.Connection.Active",
        assume_defaults = true
    )]
    pub(crate) trait ActiveConnection {
        #[zbus(property)]
        fn type_(&self) -> zbus::Result<String>;

        #[zbus(property)]
        fn devices(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
    }
}

pub(crate) struct PrimaryConnection {
    path: Option<OwnedObjectPath>,
    pub(crate) primary_device: PrimaryDevice,
}

const PRIMARY_CONNECTION_CHANGED: &str = "PRIMARY_CONNECTION_CHANGED";
const PRIMARY_DEVICES_CHANGED: &str = "PRIMARY_DEVICES_CHANGED";

impl PrimaryConnection {
    pub(crate) async fn new(conn: &Connection, stream_map: &mut StreamMap) -> Result<Self> {
        let nm = dbus::NetworkManagerProxy::new(conn).await?;

        let primary_connection = nm.primary_connection().await.ok();
        let primary_connection_sub =
            nm.receive_primary_connection_changed()
                .await
                .filter_map(|e| async move {
                    let path = e.get().await.ok()?;
                    Some(NetworkManagerEvent::PrimaryConnection(path))
                });

        stream_map.add(PRIMARY_CONNECTION_CHANGED, primary_connection_sub);
        if let Some(path) = primary_connection {
            stream_map.emit(NetworkManagerEvent::PrimaryConnection(path))?;
        }

        Ok(Self {
            path: None,
            primary_device: PrimaryDevice::new(),
        })
    }

    pub(crate) fn clear(&mut self, stream_map: &mut StreamMap) {
        self.path = None;
        stream_map.remove(PRIMARY_DEVICES_CHANGED);
        self.primary_device.clear(stream_map)
    }

    pub(crate) async fn changed(
        &mut self,
        path: OwnedObjectPath,
        conn: &Connection,
        stream_map: &mut StreamMap,
    ) -> Result<()> {
        if self.path.as_ref().is_some_and(|v| v == &path) {
            return Ok(());
        }

        self.clear(stream_map);

        let connection = dbus::ActiveConnectionProxy::builder(conn)
            .path(path.clone())?
            .build()
            .await?;
        if !connection.type_().await?.contains("wireless") {
            log::error!(target: "Network", "primary connection is not wireless");
            return Ok(());
        }

        self.path = Some(path.clone());

        let devices = connection.devices().await?;
        let devices_sub = connection
            .receive_devices_changed()
            .await
            .filter_map(|e| async move {
                let devices = e.get().await.ok()?;
                Some(NetworkManagerEvent::PrimaryDevices(devices))
            });

        stream_map.emit(NetworkManagerEvent::PrimaryDevices(devices))?;
        stream_map.add(PRIMARY_DEVICES_CHANGED, devices_sub);

        Ok(())
    }
}
