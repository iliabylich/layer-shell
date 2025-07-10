use crate::nm_event::NetworkManagerEvent;
use anyhow::Result;
use futures::{Stream, StreamExt};
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

impl AccessPointSsid {
    pub(crate) async fn stream(
        conn: &Connection,
        path: OwnedObjectPath,
    ) -> Result<impl Stream<Item = NetworkManagerEvent> + 'static> {
        let proxy = AccessPointProxy::builder(conn).path(path)?.build().await?;

        let pre = match proxy.ssid().await {
            Ok(ssid) => {
                let event = NetworkManagerEvent::Ssid(ssid);
                futures::stream::once(async move { event }).boxed()
            }
            Err(err) => {
                log::error!(target: "Network", "{err:?}");
                futures::stream::empty().boxed()
            }
        };

        let post = proxy
            .receive_ssid_changed()
            .await
            .filter_map(|e| async move {
                let ssid = e.get().await.ok()?;
                Some(NetworkManagerEvent::Ssid(ssid))
            });

        Ok(pre.chain(post))
    }
}
