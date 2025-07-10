use crate::{multiplexer::StreamId, nm_event::NetworkManagerEvent, nm_stream::NmStream};
use anyhow::Result;
use futures::{StreamExt, stream::BoxStream};
use zbus::{Connection, proxy};

#[proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
trait NetworkManager {
    #[zbus(property)]
    fn devices(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
}

pub(crate) struct GlobalDevices;

#[async_trait::async_trait]
impl NmStream for GlobalDevices {
    const ID: &StreamId = &StreamId {
        name: "GLOBAL_DEVICES",
        children: &[],
    };

    type Input = ();

    async fn stream(
        conn: &Connection,
        _: Self::Input,
    ) -> Result<BoxStream<'static, NetworkManagerEvent>> {
        let proxy = NetworkManagerProxy::new(conn).await?;

        let devices = proxy.devices().await?;
        let event = NetworkManagerEvent::Devices(devices);
        let pre = futures::stream::once(async move { event });

        let post = proxy
            .receive_devices_changed()
            .await
            .filter_map(|e| async move {
                let paths = e.get().await.ok()?;
                Some(NetworkManagerEvent::Devices(paths))
            });

        Ok(pre.chain(post).boxed())
    }
}
