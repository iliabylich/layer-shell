use crate::{multiplexer::StreamId, nm_event::NetworkManagerEvent, nm_stream::NmStream};
use anyhow::Result;
use futures::{StreamExt, stream::BoxStream};
use zbus::{Connection, proxy, zvariant::OwnedObjectPath};

#[proxy(
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager.AccessPoint",
    assume_defaults = true
)]
pub trait AccessPoint {
    #[zbus(property)]
    fn ssid(&self) -> zbus::Result<Vec<u8>>;
}

pub(crate) struct AccessPointSsid;

#[async_trait::async_trait]
impl NmStream for AccessPointSsid {
    const ID: &StreamId = &StreamId {
        name: "ACCESS_POINT_SSID",
        children: &[],
    };

    type Input = OwnedObjectPath;

    async fn stream(
        conn: &Connection,
        path: OwnedObjectPath,
    ) -> Result<BoxStream<'static, NetworkManagerEvent>> {
        let proxy = AccessPointProxy::builder(conn).path(path)?.build().await?;

        let pre = proxy
            .ssid()
            .await
            .map(|ssid| {
                let event = NetworkManagerEvent::Ssid(ssid);
                futures::stream::once(async move { event }).boxed()
            })
            .unwrap_or_else(|err| {
                log::error!(target: "Network", "{err:?}");
                futures::stream::empty().boxed()
            });

        let post = proxy
            .receive_ssid_changed()
            .await
            .filter_map(|e| async move {
                let ssid = e.get().await.ok()?;
                Some(NetworkManagerEvent::Ssid(ssid))
            });

        Ok(pre.chain(post).boxed())
    }
}
