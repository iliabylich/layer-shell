use crate::{
    NetworkData, NetworkListEvent, NetworkSsidEvent, NetworkStrengthEvent,
    access_point::AccessPoint,
    access_point_ssid::AccessPointSsid,
    access_point_strength::AccessPointStrength,
    device_rx::DeviceRx,
    device_tx::DeviceTx,
    event::NetworkEvent,
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
use module::Module;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;
use zbus::Connection;

pub struct Network {
    etx: UnboundedSender<NetworkEvent>,
    token: CancellationToken,
    stream_map: StreamMap,
    speed: Speed,
}

#[async_trait::async_trait]
impl Module for Network {
    const NAME: &str = "Network";

    type Event = NetworkEvent;
    type Command = ();
    type Ctl = ();

    fn new(
        etx: UnboundedSender<Self::Event>,
        _: UnboundedReceiver<Self::Command>,
        token: CancellationToken,
    ) -> Self {
        Self {
            etx,
            token,
            stream_map: StreamMap::new(),
            speed: Speed::new(),
        }
    }

    async fn start(&mut self) -> Result<()> {
        let conn = Connection::system().await?;

        self.stream_map
            .add(PRIMARY_CONNECTION, PrimaryConnection::stream(&conn).await?);
        self.stream_map
            .add(GLOBAL_DEVICES, GlobalDevices::stream(&conn).await?);

        loop {
            tokio::select! {
                Some((_name, event)) = self.stream_map.next() => {
                    if let Err(err) = self.on_event(event, &conn).await {
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
}

impl Network {
    async fn on_event(&mut self, event: NetworkManagerEvent, conn: &Connection) -> Result<()> {
        match event {
            NetworkManagerEvent::PrimaryConnection(path) => {
                self.stream_map.remove_with_children(PRIMARY_DEVICES);

                if PrimaryConnection::is_wireless(path.clone(), conn).await? {
                    self.stream_map
                        .add(PRIMARY_DEVICES, PrimaryDevices::stream(conn, path).await?);
                };
            }

            NetworkManagerEvent::PrimaryDevices(paths) => {
                self.stream_map.remove_with_children(ACCESS_POINT);
                self.stream_map.remove_with_children(DEVICE_RX);
                self.stream_map.remove_with_children(DEVICE_TX);
                self.speed.reset();

                let path = sole(paths)?;

                self.stream_map
                    .add(ACCESS_POINT, AccessPoint::stream(conn, path.clone()).await?);

                self.stream_map
                    .add(DEVICE_RX, DeviceRx::stream(conn, path.clone()).await?);

                self.stream_map
                    .add(DEVICE_TX, DeviceTx::stream(conn, path.clone()).await?);
            }

            NetworkManagerEvent::AccessPoint(path) => {
                self.stream_map.remove_with_children(ACCESS_POINT_SSID);
                self.stream_map.remove_with_children(ACCESS_POINT_STRENGTH);

                self.stream_map.add(
                    ACCESS_POINT_SSID,
                    AccessPointSsid::stream(conn, path.clone()).await?,
                );

                self.stream_map.add(
                    ACCESS_POINT_STRENGTH,
                    AccessPointStrength::stream(conn, path.clone()).await?,
                );
            }

            NetworkManagerEvent::Ssid(ssid) => {
                let ssid = String::from_utf8_lossy(&ssid).to_string();
                let event = NetworkEvent::Ssid(NetworkSsidEvent { ssid: ssid.into() });
                self.etx.send(event)?;
            }
            NetworkManagerEvent::Strength(strength) => {
                let event = NetworkEvent::Strength(NetworkStrengthEvent { strength });
                self.etx.send(event)?;
            }

            NetworkManagerEvent::Devices(paths) => {
                let networks = NetworkData::read_many(conn, paths).await;
                let event = NetworkEvent::NetworkList(NetworkListEvent {
                    list: networks.into(),
                });
                self.etx.send(event)?;
            }

            NetworkManagerEvent::DeviceTxBytes(tx) => {
                let event = self.speed.update_tx(tx);
                self.etx.send(event)?;
            }
            NetworkManagerEvent::DeviceRxBytes(rx) => {
                let event = self.speed.update_rx(rx);
                self.etx.send(event)?;
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
