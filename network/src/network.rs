use crate::{
    devices::Devices, event::Event, nm_event::NetworkManagerEvent,
    primary_connection::PrimaryConnection, stream_map::StreamMap,
};
use anyhow::{Result, bail};
use utils::{TaskCtx, service};
use zbus::Connection;

struct Task {
    ctx: TaskCtx<Event>,
    conn: Connection,

    stream_map: StreamMap,

    primary_connection: PrimaryConnection,
}

impl Task {
    async fn start(ctx: TaskCtx<Event>) -> Result<()> {
        let conn = Connection::system().await?;

        let mut stream_map = StreamMap::new();

        let primary_connection = PrimaryConnection::new(&conn, &mut stream_map).await?;
        Devices::subscribe(&conn, &mut stream_map).await?;

        Self {
            ctx,
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

                _ = &mut self.ctx.exit => {
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
                    self.ctx.emitter.emit(event)?;
                }
            }
            NetworkManagerEvent::Strength(strength) => {
                if let Some(event) = self
                    .primary_connection
                    .primary_device
                    .access_point
                    .strength_changed(strength)
                {
                    self.ctx.emitter.emit(event)?;
                }
            }
            NetworkManagerEvent::Devices(paths) => {
                let event = Devices::changed(paths, &self.conn).await;
                self.ctx.emitter.emit(event)?;
            }
            NetworkManagerEvent::DeviceTxBytes(tx) => {
                let event = self.primary_connection.primary_device.speed.update_tx(tx);
                self.ctx.emitter.emit(event)?;
            }
            NetworkManagerEvent::DeviceRxBytes(rx) => {
                let event = self.primary_connection.primary_device.speed.update_rx(rx);
                self.ctx.emitter.emit(event)?;
            }
        }

        Ok(())
    }
}

service!(Network, Event, Task::start);

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
