use crate::{
    access_point::AccessPoint, device_rx::DeviceRx, device_tx::DeviceTx, multiplexer::StreamId,
    nm_event::NetworkManagerEvent, nm_stream::NmStream,
};
use anyhow::Result;
use futures::{StreamExt as _, stream::BoxStream};
use zbus::zvariant::OwnedObjectPath;
use zbus::{Connection, proxy};

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

#[async_trait::async_trait]
impl NmStream for PrimaryDevices {
    const ID: &StreamId = &StreamId {
        name: "PRIMARY_DEVICES",
        children: &[AccessPoint::ID, DeviceTx::ID, DeviceRx::ID],
    };

    type Input = OwnedObjectPath;

    async fn stream(
        conn: &Connection,
        path: OwnedObjectPath,
    ) -> Result<BoxStream<'static, NetworkManagerEvent>> {
        let connection = ActiveConnectionProxy::builder(conn)
            .path(path.clone())?
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

        Ok(pre.chain(post).boxed())
    }
}
