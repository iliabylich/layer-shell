use crate::{MuteChangedEvent, SoundEvent, VolumeChangedEvent, dbus::PipewireDBusProxy};
use anyhow::{Context as _, Result, bail};
use futures::StreamExt;
use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;
use zbus::Connection;

pub(crate) struct SoundTask {
    tx: UnboundedSender<SoundEvent>,
    token: CancellationToken,

    conn: Connection,
}

impl SoundTask {
    pub(crate) async fn start(
        tx: UnboundedSender<SoundEvent>,
        token: CancellationToken,
    ) -> Result<()> {
        let conn = Connection::session().await?;

        Self { tx, token, conn }.r#loop().await
    }

    async fn r#loop(mut self) -> Result<()> {
        let proxy = PipewireDBusProxy::new(&self.conn)
            .await
            .context("failed to create pipewire-dbus proxy")?;

        let mut volume_stream = proxy
            .receive_volume_changed()
            .await
            .filter_map(async move |e| {
                let volume = e.get().await.ok()?;
                Some(SoundEvent::VolumeChangedEvent(VolumeChangedEvent {
                    volume,
                }))
            })
            .boxed();

        let mut muted_stream = proxy
            .receive_muted_changed()
            .await
            .filter_map(async move |e| {
                let muted = e.get().await.ok()?;
                Some(SoundEvent::MuteChangedEvent(MuteChangedEvent { muted }))
            })
            .boxed();

        loop {
            tokio::select! {
                Some(event) = volume_stream.next() => {
                    self.emit(event)?;
                }

                Some(event) = muted_stream.next() => {
                    self.emit(event)?;
                }

                _ = self.token.cancelled() => {
                    log::info!(target: "Network", "exiting...");
                    return Ok(())
                }

                else => bail!("all streams are closed")
            }
        }
    }

    fn emit(&mut self, event: SoundEvent) -> Result<()> {
        self.tx
            .send(event)
            .context("failed to send Sound event: channel is closed")
    }
}
