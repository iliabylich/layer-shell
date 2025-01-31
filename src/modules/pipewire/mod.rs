use std::time::Duration;

use crate::{
    dbus::{
        OrgLocalPipewireDBus, OrgLocalPipewireDBusMutedUpdated, OrgLocalPipewireDBusVolumeUpdated,
    },
    scheduler::{Module, RepeatingModule},
    Command, Event,
};
use anyhow::{Context as _, Result};
use dbus::{blocking::Connection, message::SignalArgs as _};

pub(crate) struct Pipewire {
    conn: Connection,
}

impl Module for Pipewire {
    const NAME: &str = "Pipewire2";

    fn start() -> Result<Option<Box<dyn RepeatingModule>>> {
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

        Ok(Some(Box::new(Self { conn })))
    }
}

impl RepeatingModule for Pipewire {
    fn tick(&mut self) -> Result<Duration> {
        while self.conn.process(Duration::from_millis(100))? {}
        Ok(Duration::from_millis(100))
    }

    fn exec(&mut self, cmd: &Command) -> Result<()> {
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

        Ok(())
    }
}
