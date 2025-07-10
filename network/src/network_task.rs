use crate::{
    NetworkData, NetworkEvent, NetworkListEvent, NetworkSsidEvent, NetworkStrengthEvent,
    access_point::AccessPoint,
    access_point_ssid::AccessPointSsid,
    access_point_strength::AccessPointStrength,
    device_rx::DeviceRx,
    device_tx::DeviceTx,
    global_devices::GlobalDevices,
    nm_event::NetworkManagerEvent,
    primary_connection::PrimaryConnection,
    primary_devices::PrimaryDevices,
    speed::Speed,
    stream_map::{
        ACCESS_POINT, ACCESS_POINT_SSID, ACCESS_POINT_STRENGTH, DEVICE_RX, DEVICE_TX,
        GLOBAL_DEVICES, PRIMARY_CONNECTION, PRIMARY_DEVICES, StreamMap,
    },
};
use anyhow::{Result, bail};
use futures::StreamExt;
use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;
use zbus::Connection;

pub(crate) struct NetworkTask {
    tx: UnboundedSender<NetworkEvent>,
    token: CancellationToken,

    conn: Connection,

    stream_map: StreamMap,

    speed: Speed,
}

impl NetworkTask {
    pub(crate) async fn start(
        tx: UnboundedSender<NetworkEvent>,
        token: CancellationToken,
    ) -> Result<()> {
        let conn = Connection::system().await?;

        let mut stream_map = StreamMap::new();

        stream_map.add(PRIMARY_CONNECTION, PrimaryConnection::stream(&conn).await?);
        stream_map.add(GLOBAL_DEVICES, GlobalDevices::stream(&conn).await?);

        Self {
            tx,
            token,

            conn,

            stream_map,

            speed: Speed::new(),
        }
        .r#loop()
        .await
    }

    async fn r#loop(mut self) -> Result<()> {
        loop {
            tokio::select! {
                Some((_name, event)) = self.stream_map.next() => {
                    if let Err(err) = self.on_event(event).await {
                        log::error!(target: "Network", "{err:?}");
                    }
                }

                _ = self.token.cancelled() => {
                    log::info!(target: "Network", "exiting...");
                    return Ok(())
                }

                else => bail!("all streams are closed")
            }
        }
    }

    async fn on_event(&mut self, event: NetworkManagerEvent) -> Result<()> {
        match event {
            NetworkManagerEvent::PrimaryConnection(path) => {
                self.stream_map.remove_with_children(PRIMARY_DEVICES);

                if PrimaryConnection::is_wireless(path.clone(), &self.conn).await? {
                    self.stream_map.add(
                        PRIMARY_DEVICES,
                        PrimaryDevices::stream(&self.conn, path).await?,
                    );
                };
            }

            NetworkManagerEvent::PrimaryDevices(paths) => {
                self.stream_map.remove_with_children(ACCESS_POINT);
                self.stream_map.remove_with_children(DEVICE_RX);
                self.stream_map.remove_with_children(DEVICE_TX);
                self.speed.reset();

                let path = sole(paths)?;

                self.stream_map.add(
                    ACCESS_POINT,
                    AccessPoint::stream(&self.conn, path.clone()).await?,
                );

                self.stream_map
                    .add(DEVICE_RX, DeviceRx::stream(&self.conn, path.clone()).await?);

                self.stream_map
                    .add(DEVICE_TX, DeviceTx::stream(&self.conn, path.clone()).await?);
            }

            NetworkManagerEvent::AccessPoint(path) => {
                self.stream_map.remove_with_children(ACCESS_POINT_SSID);
                self.stream_map.remove_with_children(ACCESS_POINT_STRENGTH);

                self.stream_map.add(
                    ACCESS_POINT_SSID,
                    AccessPointSsid::stream(&self.conn, path.clone()).await?,
                );

                self.stream_map.add(
                    ACCESS_POINT_STRENGTH,
                    AccessPointStrength::stream(&self.conn, path.clone()).await?,
                );
            }

            NetworkManagerEvent::Ssid(ssid) => {
                let ssid = String::from_utf8_lossy(&ssid).to_string();
                let event = NetworkEvent::Ssid(NetworkSsidEvent { ssid: ssid.into() });
                self.tx.send(event)?;
            }
            NetworkManagerEvent::Strength(strength) => {
                let event = NetworkEvent::Strength(NetworkStrengthEvent { strength });
                self.tx.send(event)?;
            }

            NetworkManagerEvent::Devices(paths) => {
                let networks = NetworkData::read_many(&self.conn, paths).await;
                let event = NetworkEvent::NetworkList(NetworkListEvent {
                    list: networks.into(),
                });
                self.tx.send(event)?;
            }

            NetworkManagerEvent::DeviceTxBytes(tx) => {
                let event = self.speed.update_tx(tx);
                self.tx.send(event)?;
            }
            NetworkManagerEvent::DeviceRxBytes(rx) => {
                let event = self.speed.update_rx(rx);
                self.tx.send(event)?;
            }
        }

        Ok(())
    }
}

fn sole<T>(list: Vec<T>) -> Result<T> {
    let mut iter = list.into_iter();

    let Some(item) = iter.next() else {
        bail!("got 0 items in vec (expected 1)");
    };
    if iter.next().is_some() {
        bail!("got multiple in vec (expected 1)");
    }
    Ok(item)
}
