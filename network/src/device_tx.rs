use crate::nm_event::NetworkManagerEvent;
use anyhow::Result;
use futures::{Stream, StreamExt as _};
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
    fn tx_bytes(&self) -> zbus::Result<u64>;
}

pub(crate) struct DeviceTx;

impl DeviceTx {
    pub(crate) async fn stream(
        conn: &Connection,
        path: OwnedObjectPath,
    ) -> Result<impl Stream<Item = NetworkManagerEvent> + 'static> {
        let proxy = DeviceStatisticsProxy::builder(conn)
            .path(path.clone())?
            .build()
            .await?;

        proxy.set_refresh_rate_ms(1_000).await?;

        let stream = proxy
            .receive_tx_bytes_changed()
            .await
            .filter_map(|e| async move {
                let tx = e.get().await.ok()?;
                Some(NetworkManagerEvent::DeviceTxBytes(tx))
            });

        Ok(stream)
    }
}
