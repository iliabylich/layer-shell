use std::{ops::ControlFlow, sync::mpsc::Sender, time::Duration};

use crate::{
    dbus::{
        OrgLocalPipewireDBus, OrgLocalPipewireDBusMutedUpdated, OrgLocalPipewireDBusVolumeUpdated,
    },
    scheduler::Actor,
    Command, Event,
};
use anyhow::{Context as _, Result};
use dbus::{blocking::Connection, message::SignalArgs as _};

pub(crate) struct Pipewire {
    conn: Connection,
}

impl Actor for Pipewire {
    fn name() -> &'static str {
        "Pipewire"
    }

    fn start(tx: Sender<Event>) -> Result<Box<dyn Actor>> {
        let conn = Connection::new_session().context("Failed to connect to D-Bus")?;

        let proxy = conn.with_proxy(
            "org.local.PipewireDBus",
            "/org/local/PipewireDBus",
            Duration::from_millis(5000),
        );
        let volume = proxy.get_volume().context("failed to call GetVolume")?;
        let muted = proxy.get_muted().context("failed to call GetMuted")?;
        tx.send(Event::Volume {
            volume: volume as f32,
        })
        .context("failed to send Volume event")?;
        tx.send(Event::Mute { muted })
            .context("failed to send Mute event")?;

        {
            let tx = tx.clone();
            conn.add_match(
                OrgLocalPipewireDBusMutedUpdated::match_rule(None, None),
                move |e: OrgLocalPipewireDBusMutedUpdated, _, _| {
                    let event = Event::Mute { muted: e.muted };
                    if let Err(err) = tx.send(event) {
                        log::error!("failed to send Mute event: {:?}", err)
                    }
                    true
                },
            )
            .context("failed to add_match")?;
        }

        {
            let tx = tx.clone();
            conn.add_match(
                OrgLocalPipewireDBusVolumeUpdated::match_rule(None, None),
                move |e: OrgLocalPipewireDBusVolumeUpdated, _, _| {
                    let event = Event::Volume {
                        volume: e.volume as f32,
                    };
                    if let Err(err) = tx.send(event) {
                        log::error!("failed to send Volume event: {:?}", err);
                    }
                    true
                },
            )
            .context("failed to add_match")?;
        }

        Ok(Box::new(Self { conn }))
    }

    fn tick(&mut self) -> Result<ControlFlow<(), Duration>> {
        while self.conn.process(Duration::from_millis(0))? {}
        Ok(ControlFlow::Continue(Duration::from_millis(100)))
    }

    fn exec(&mut self, cmd: &Command) -> Result<ControlFlow<()>> {
        let proxy = self.conn.with_proxy(
            "org.local.PipewireDBus",
            "/org/local/PipewireDBus",
            Duration::from_millis(5000),
        );

        match cmd {
            Command::SetVolume { volume } => {
                proxy
                    .set_volume(*volume as f64)
                    .context("failed to call SetVolume")?;
            }
            Command::SetMuted { muted } => {
                proxy.set_muted(*muted).context("failed to call SetMuted")?;
            }

            _ => {}
        }

        Ok(ControlFlow::Continue(()))
    }
}

impl std::fmt::Debug for Pipewire {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pipewire").field("conn", &"<conn>").finish()
    }
}
