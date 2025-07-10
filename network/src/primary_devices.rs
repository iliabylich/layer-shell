use crate::nm_event::NetworkManagerEvent;
use anyhow::Result;
use futures::{Stream, StreamExt as _};
use zbus::proxy;
use zbus::zvariant::OwnedObjectPath;

#[proxy(
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager.Connection.Active",
    assume_defaults = true
)]
trait ActiveConnection {
    #[zbus(property)]
    fn devices(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
}

pub(crate) struct PrimaryDevices;

impl PrimaryDevices {
    pub(crate) async fn stream(
        conn: &zbus::Connection,
        primary_connection_path: OwnedObjectPath,
    ) -> Result<impl Stream<Item = NetworkManagerEvent> + 'static> {
        let connection = ActiveConnectionProxy::builder(conn)
            .path(primary_connection_path.clone())?
            .build()
            .await?;

        let devices = connection.devices().await?;
        let event = NetworkManagerEvent::PrimaryDevices(devices);
        let pre = futures::stream::once(async move { event });

        let post = connection
            .receive_devices_changed()
            .await
            .filter_map(|e| async move {
                let devices = e.get().await.ok()?;
                Some(NetworkManagerEvent::PrimaryDevices(devices))
            });

        Ok(pre.chain(post))
    }
}
