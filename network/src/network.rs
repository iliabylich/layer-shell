use crate::{
    NetworkData, NetworkListEvent, NetworkSsidEvent, NetworkStrengthEvent,
    access_point::AccessPoint, access_point_ssid::AccessPointSsid,
    access_point_strength::AccessPointStrength, device_rx::DeviceRx, device_tx::DeviceTx,
    event::NetworkEvent, global_devices::GlobalDevices, multiplexer::Multiplexer,
    nm_event::NetworkManagerEvent, nm_stream::NmStream, primary_connection::PrimaryConnection,
    primary_devices::PrimaryDevices, speed::Speed,
};
use anyhow::{Result, bail};
use futures::StreamExt;
use module::{Module, TimerSubscriber};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;
use zbus::Connection;

pub struct Network {
    etx: UnboundedSender<NetworkEvent>,
    token: CancellationToken,
    multiplexer: Multiplexer,
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
        _: TimerSubscriber,
    ) -> Self {
        Self {
            etx,
            token,
            multiplexer: Multiplexer::new(),
            speed: Speed::new(),
        }
    }

    async fn start(&mut self) -> Result<()> {
        let conn = Connection::system().await?;

        self.add::<PrimaryConnection>(&conn, ()).await?;
        self.add::<GlobalDevices>(&conn, ()).await?;

        loop {
            tokio::select! {
                Some((_name, event)) = self.multiplexer.next() => {
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
    async fn add<S: NmStream>(&mut self, conn: &Connection, input: S::Input) -> Result<()> {
        let stream = S::stream(conn, input).await?;
        self.multiplexer.add(S::ID, stream);
        Ok(())
    }
    fn remove<S: NmStream>(&mut self) {
        self.multiplexer.remove_with_children(S::ID);
    }

    async fn on_event(&mut self, event: NetworkManagerEvent, conn: &Connection) -> Result<()> {
        match event {
            NetworkManagerEvent::PrimaryConnection(path) => {
                self.remove::<PrimaryDevices>();

                if PrimaryConnection::is_wireless(path.clone(), conn).await? {
                    self.add::<PrimaryDevices>(conn, path).await?;
                };
            }

            NetworkManagerEvent::PrimaryDevices(paths) => {
                self.remove::<AccessPoint>();
                self.remove::<DeviceRx>();
                self.remove::<DeviceTx>();
                self.speed.reset();

                let path = sole(paths)?;

                self.add::<AccessPoint>(conn, path.clone()).await?;
                self.add::<DeviceRx>(conn, path.clone()).await?;
                self.add::<DeviceTx>(conn, path.clone()).await?;
            }

            NetworkManagerEvent::AccessPoint(path) => {
                self.remove::<AccessPointSsid>();
                self.remove::<AccessPointStrength>();

                self.add::<AccessPointSsid>(conn, path.clone()).await?;
                self.add::<AccessPointStrength>(conn, path.clone()).await?;
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
