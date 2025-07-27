use crate::{
    InitialSoundEvent, MuteChangedEvent, SoundEvent, VolumeChangedEvent, dbus::PipewireDBusProxy,
};
use anyhow::{Context as _, Result, bail};
use futures::StreamExt as _;
use module::{Module, TimerSubscriber};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;
use zbus::Connection;

pub struct Sound {
    etx: UnboundedSender<SoundEvent>,
    token: CancellationToken,
}

#[async_trait::async_trait]
impl Module for Sound {
    const NAME: &str = "Sound";
    type Event = SoundEvent;
    type Command = ();
    type Ctl = ();

    fn new(
        etx: UnboundedSender<Self::Event>,
        _: UnboundedReceiver<Self::Command>,
        token: CancellationToken,
        _: TimerSubscriber,
    ) -> Self {
        Self { etx, token }
    }

    async fn start(&mut self) -> Result<()> {
        let conn = Connection::session().await?;

        let proxy = PipewireDBusProxy::new(&conn)
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
}

impl Sound {
    fn emit(&mut self, event: SoundEvent) -> Result<()> {
        self.etx
            .send(event)
            .context("failed to send Sound event: channel is closed")
    }
}
