use crate::{
    access_point::AccessPoint, nm_event::NetworkManagerEvent, speed::Speed, stream_map::StreamMap,
};
use anyhow::Result;
use futures::StreamExt;
use zbus::{Connection, zvariant::OwnedObjectPath};

mod dbus {
    use zbus::proxy;

    #[proxy(
        default_service = "org.freedesktop.NetworkManager",
        interface = "org.freedesktop.NetworkManager.Device.Wireless",
        assume_defaults = true
    )]
    pub(crate) trait WirelessDevice {
        #[zbus(property)]
        fn active_access_point(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
    }

    #[proxy(
        default_service = "org.freedesktop.NetworkManager",
        interface = "org.freedesktop.NetworkManager.Device.Statistics",
        assume_defaults = true
    )]
    pub(crate) trait DeviceStatistics {
        #[zbus(property)]
        fn set_refresh_rate_ms(&self, value: u32) -> zbus::Result<()>;

        #[zbus(property)]
        fn rx_bytes(&self) -> zbus::Result<u64>;

        #[zbus(property)]
        fn tx_bytes(&self) -> zbus::Result<u64>;
    }
}

pub(crate) struct PrimaryDevice {
    path: Option<OwnedObjectPath>,
    pub(crate) access_point: AccessPoint,
    pub(crate) speed: Speed,
}

const ACCESS_POINT_CHANGED: &str = "ACCESS_POINT_CHANGED";
const DEVICE_TX_BYTES_CHANGED: &str = "DEVICE_TX_BYTES_CHANGED";
const DEVICE_RX_BYTES_CHANGED: &str = "DEVICE_RX_BYTES_CHANGED";

impl PrimaryDevice {
    pub(crate) fn new() -> Self {
        Self {
            path: None,
            access_point: AccessPoint::new(),
            speed: Speed::new(),
        }
    }

    pub(crate) fn clear(&mut self, stream_map: &mut StreamMap) {
        self.path = None;
        stream_map.remove(ACCESS_POINT_CHANGED);
        stream_map.remove(DEVICE_TX_BYTES_CHANGED);
        stream_map.remove(DEVICE_RX_BYTES_CHANGED);
        self.access_point.clear(stream_map);
        self.speed.reset();
    }

    pub(crate) async fn changed(
        &mut self,
        path: OwnedObjectPath,
        conn: &Connection,
        stream_map: &mut StreamMap,
    ) -> Result<()> {
        if self.path.as_ref().is_some_and(|v| v == &path) {
            return Ok(());
        }

        self.clear(stream_map);

        let device = dbus::WirelessDeviceProxy::builder(conn)
            .path(path.clone())?
            .build()
            .await?;

        let stats = dbus::DeviceStatisticsProxy::builder(conn)
            .path(path.clone())?
            .build()
            .await?;

        self.path = Some(path);

        stats.set_refresh_rate_ms(1_000).await?;

        let access_point = device.active_access_point().await?;
        let access_point_sub = device
            .receive_active_access_point_changed()
            .await
            .filter_map(|e| async move {
                let path = e.get().await.ok()?;
                Some(NetworkManagerEvent::AccessPoint(path))
            });

        let tx_bytes_sub = stats
            .receive_tx_bytes_changed()
            .await
            .filter_map(|e| async move {
                let tx = e.get().await.ok()?;
                Some(NetworkManagerEvent::DeviceTxBytes(tx))
            });

        let rx_bytes_sub = stats
            .receive_rx_bytes_changed()
            .await
            .filter_map(|e| async move {
                let rx = e.get().await.ok()?;
                Some(NetworkManagerEvent::DeviceRxBytes(rx))
            });

        stream_map.emit(NetworkManagerEvent::AccessPoint(access_point))?;

        stream_map.add(ACCESS_POINT_CHANGED, access_point_sub);
        stream_map.add(DEVICE_TX_BYTES_CHANGED, tx_bytes_sub);
        stream_map.add(DEVICE_RX_BYTES_CHANGED, rx_bytes_sub);

        Ok(())
    }
}
