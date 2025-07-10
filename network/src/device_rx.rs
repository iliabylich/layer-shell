use crate::{multiplexer::StreamId, nm_event::NetworkManagerEvent, nm_stream::NmStream};
use anyhow::Result;
use futures::{StreamExt as _, stream::BoxStream};
use zbus::{Connection, proxy, zvariant::OwnedObjectPath};

#[proxy(
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager.Device.Statistics",
    assume_defaults = true
)]
trait DeviceStatistics {
    #[zbus(property)]
    fn set_refresh_rate_ms(&self, value: u32) -> zbus::Result<()>;

    #[zbus(property)]
    fn rx_bytes(&self) -> zbus::Result<u64>;
}

pub(crate) struct DeviceRx;

#[async_trait::async_trait]
impl NmStream for DeviceRx {
    const ID: &StreamId = &StreamId {
        name: "DEVICE_RX",
        children: &[],
    };

    type Input = OwnedObjectPath;

    async fn stream(
        conn: &Connection,
        path: OwnedObjectPath,
    ) -> Result<BoxStream<'static, NetworkManagerEvent>> {
        let proxy = DeviceStatisticsProxy::builder(conn)
            .path(path.clone())?
            .build()
            .await?;

        proxy.set_refresh_rate_ms(1_000).await?;

        let stream = proxy
            .receive_rx_bytes_changed()
            .await
            .filter_map(|e| async move {
                let rx = e.get().await.ok()?;
                Some(NetworkManagerEvent::DeviceRxBytes(rx))
            });

        Ok(stream.boxed())
    }
}
