use crate::nm_event::NetworkManagerEvent;
use anyhow::Result;
use futures::{Stream, StreamExt};
use zbus::proxy;

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

impl GlobalDevices {
    pub(crate) async fn stream(
        conn: &zbus::Connection,
    ) -> Result<impl Stream<Item = NetworkManagerEvent> + 'static> {
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

        Ok(pre.chain(post))
    }
}
