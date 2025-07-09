use crate::{
    InitialSoundEvent, MuteChangedEvent, SoundEvent, VolumeChangedEvent, dbus::PipewireDBusProxy,
};
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
            .skip(1)
            .filter_map(async move |e| e.get().await.ok())
            .boxed();

        let mut muted_stream = proxy
            .receive_muted_changed()
            .await
            .skip(1)
            .filter_map(async move |e| e.get().await.ok())
            .boxed();

        let volume = proxy
            .volume()
            .await
            .context("failed to get initial 'volume'")?;
        let muted = proxy
            .muted()
            .await
            .context("failed to receive initial 'muted'")?;
        self.emit(SoundEvent::InitialSoundEvent(InitialSoundEvent {
            volume,
            muted,
        }))?;

        loop {
            tokio::select! {
                Some(volume) = volume_stream.next() => {
                    self.emit(SoundEvent::VolumeChangedEvent(VolumeChangedEvent { volume }))?;
                }

                Some(muted) = muted_stream.next() => {
                    self.emit(SoundEvent::MuteChangedEvent(MuteChangedEvent { muted }))?;
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
