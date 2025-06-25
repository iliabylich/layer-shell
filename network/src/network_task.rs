use crate::{
    NetworkEvent, devices::Devices, nm_event::NetworkManagerEvent,
    primary_connection::PrimaryConnection, stream_map::StreamMap,
};
use anyhow::{Result, bail};
use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;
use zbus::Connection;

pub(crate) struct NetworkTask {
    tx: UnboundedSender<NetworkEvent>,
    token: CancellationToken,

    conn: Connection,

    stream_map: StreamMap,

    primary_connection: PrimaryConnection,
}

impl NetworkTask {
    pub(crate) async fn start(
        tx: UnboundedSender<NetworkEvent>,
        token: CancellationToken,
    ) -> Result<()> {
        let conn = Connection::system().await?;

        let mut stream_map = StreamMap::new();

        let primary_connection = PrimaryConnection::new(&conn, &mut stream_map).await?;
        Devices::subscribe(&conn, &mut stream_map).await?;

        Self {
            tx,
            token,

            conn,

            stream_map,

            primary_connection,
        }
        .r#loop()
        .await
    }

    async fn r#loop(mut self) -> Result<()> {
        loop {
            tokio::select! {
                Some((_name, event)) = self.stream_map.next() => {
                    self.on_event(event).await?;
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
                self.primary_connection
                    .changed(path, &self.conn, &mut self.stream_map)
                    .await?;
            }
            NetworkManagerEvent::PrimaryDevices(paths) => {
                let Some(path) = sole(paths) else {
                    return Ok(());
                };
                self.primary_connection
                    .primary_device
                    .changed(path, &self.conn, &mut self.stream_map)
                    .await?;
            }
            NetworkManagerEvent::AccessPoint(path) => {
                self.primary_connection
                    .primary_device
                    .access_point
                    .changed(path, &self.conn, &mut self.stream_map)
                    .await?;
            }
            NetworkManagerEvent::Ssid(ssid) => {
                if let Some(event) = self
                    .primary_connection
                    .primary_device
                    .access_point
                    .ssid_changed(ssid)
                {
                    self.tx.send(event)?;
                }
            }
            NetworkManagerEvent::Strength(strength) => {
                if let Some(event) = self
                    .primary_connection
                    .primary_device
                    .access_point
                    .strength_changed(strength)
                {
                    self.tx.send(event)?;
                }
            }
            NetworkManagerEvent::Devices(paths) => {
                let event = Devices::changed(paths, &self.conn).await;
                self.tx.send(event)?;
            }
            NetworkManagerEvent::DeviceTxBytes(tx) => {
                let event = self.primary_connection.primary_device.speed.update_tx(tx);
                self.tx.send(event)?;
            }
            NetworkManagerEvent::DeviceRxBytes(rx) => {
                let event = self.primary_connection.primary_device.speed.update_rx(rx);
                self.tx.send(event)?;
            }
        }

        Ok(())
    }
}

fn sole<T>(list: Vec<T>) -> Option<T> {
    let mut iter = list.into_iter();

    let Some(item) = iter.next() else {
        log::error!("got 0 items in vec (expected 1)");
        return None;
    };
    if iter.next().is_some() {
        log::error!("got multiple in vec (expected 1)");
        return None;
    }
    Some(item)
}
