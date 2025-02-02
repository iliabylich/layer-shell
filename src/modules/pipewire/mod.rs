use std::{ops::ControlFlow, time::Duration};

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

    fn start() -> Result<Box<dyn Actor>> {
        let conn = Connection::new_session().context("Failed to connect to D-Bus")?;

        let proxy = conn.with_proxy(
            "org.local.PipewireDBus",
            "/org/local/PipewireDBus",
            Duration::from_millis(5000),
        );
        let volume = proxy.get_volume().context("failed to call GetVolume")?;
        let muted = proxy.get_muted().context("failed to call GetMuted")?;
        Event::Volume {
            volume: volume as f32,
        }
        .emit();
        Event::Mute { muted }.emit();

        conn.add_match(
            OrgLocalPipewireDBusMutedUpdated::match_rule(None, None),
            |e: OrgLocalPipewireDBusMutedUpdated, _, _| {
                Event::Mute { muted: e.muted }.emit();
                true
            },
        )
        .context("failed to add_match")?;

        conn.add_match(
            OrgLocalPipewireDBusVolumeUpdated::match_rule(None, None),
            |e: OrgLocalPipewireDBusVolumeUpdated, _, _| {
                Event::Volume {
                    volume: e.volume as f32,
                }
                .emit();

                true
            },
        )
        .context("failed to add_match")?;

        Ok(Box::new(Self { conn }))
    }

    fn tick(&mut self) -> Result<ControlFlow<(), Duration>> {
        while self.conn.process(Duration::from_millis(100))? {}
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
