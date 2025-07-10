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
    fn strength(&self) -> zbus::Result<u8>;
}

pub(crate) struct AccessPointStrength;

#[async_trait::async_trait]
impl NmStream for AccessPointStrength {
    const ID: &StreamId = &StreamId {
        name: "ACCESS_POINT_STRENGTH",
        children: &[],
    };

    type Input = OwnedObjectPath;

    async fn stream(
        conn: &Connection,
        path: OwnedObjectPath,
    ) -> Result<BoxStream<'static, NetworkManagerEvent>> {
        let proxy = AccessPointProxy::builder(conn).path(path)?.build().await?;

        let pre = match proxy.strength().await {
            Ok(strength) => {
                let event = NetworkManagerEvent::Strength(strength);
                futures::stream::once(async move { event }).boxed()
            }
            Err(err) => {
                log::error!(target: "Network", "{err:?}");
                futures::stream::empty().boxed()
            }
        };

        let post = proxy
            .receive_strength_changed()
            .await
            .filter_map(|e| async move {
                let strength = e.get().await.ok()?;
                Some(NetworkManagerEvent::Strength(strength))
            });

        Ok(pre.chain(post).boxed())
    }
}
