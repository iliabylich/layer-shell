use crate::{
    access_point_ssid::AccessPointSsid, access_point_strength::AccessPointStrength,
    multiplexer::StreamId, nm_event::NetworkManagerEvent, nm_stream::NmStream,
};
use anyhow::Result;
use futures::{StreamExt, stream::BoxStream};
use zbus::proxy;
use zbus::{Connection, zvariant::OwnedObjectPath};

#[proxy(
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager.Device.Wireless",
    assume_defaults = true
)]
trait WirelessDevice {
    #[zbus(property)]
    fn active_access_point(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}

pub(crate) struct AccessPoint;

#[async_trait::async_trait]
impl NmStream for AccessPoint {
    const ID: &StreamId = &StreamId {
        name: "ACCESS_POINT",
        children: &[AccessPointSsid::ID, AccessPointStrength::ID],
    };

    type Input = OwnedObjectPath;

    async fn stream(
        conn: &Connection,
        path: OwnedObjectPath,
    ) -> Result<BoxStream<'static, NetworkManagerEvent>> {
        let proxy = WirelessDeviceProxy::builder(conn)
            .path(path)?
            .build()
            .await?;

        let pre = proxy
            .active_access_point()
            .await
            .map(|path| {
                let event = NetworkManagerEvent::AccessPoint(path);
                futures::stream::once(async move { event }).boxed()
            })
            .unwrap_or_else(|err| {
                log::error!(target: "Network", "{err:?}");
                futures::stream::empty().boxed()
            });

        let post = proxy
            .receive_active_access_point_changed()
            .await
            .filter_map(|e| async move {
                let path = e.get().await.ok()?;
                Some(NetworkManagerEvent::AccessPoint(path))
            });

        Ok(pre.chain(post).boxed())
    }
}
