use crate::{
    dbus::{
        OrgLocalPipewireDBus, OrgLocalPipewireDBusMutedUpdated, OrgLocalPipewireDBusVolumeUpdated,
    },
    Event, VerboseSender,
};
use anyhow::{Context as _, Result};
use dbus::{
    blocking::Connection,
    channel::{BusType, Channel},
    message::SignalArgs as _,
};
use std::{os::fd::AsRawFd, time::Duration};

pub(crate) struct Pipewire {
    tx: VerboseSender<Event>,
    conn: Connection,
}

impl Pipewire {
    pub(crate) fn new(tx: VerboseSender<Event>) -> Result<Self> {
        let mut channel =
            Channel::get_private(BusType::Session).context("failed to connect to DBus")?;
        channel.set_watch_enabled(true);
        let conn = Connection::from(channel);

        let proxy = conn.with_proxy(
            "org.local.PipewireDBus",
            "/org/local/PipewireDBus",
            Duration::from_millis(1000),
        );
        let volume = proxy.get_volume().context("failed to call GetVolume")?;
        let muted = proxy.get_muted().context("failed to call GetMuted")?;
        tx.send(Event::Volume { volume });
        tx.send(Event::Mute { muted });

        conn.add_match(
            OrgLocalPipewireDBusMutedUpdated::match_rule(None, None),
            |_: OrgLocalPipewireDBusMutedUpdated, _, _| true,
        )
        .context("failed to add_match")?;

        conn.add_match(
            OrgLocalPipewireDBusVolumeUpdated::match_rule(None, None),
            |_: OrgLocalPipewireDBusVolumeUpdated, _, _| true,
        )
        .context("failed to add_match")?;

        Ok(Self { tx, conn })
    }

    pub(crate) fn read(&mut self) {
        while let Ok(Some(message)) = self
            .conn
            .channel()
            .blocking_pop_message(Duration::from_secs(0))
        {
            if let Some(e) = OrgLocalPipewireDBusMutedUpdated::from_message(&message) {
                self.tx.send(Event::Mute { muted: e.muted });
            } else if let Some(e) = OrgLocalPipewireDBusVolumeUpdated::from_message(&message) {
                self.tx.send(Event::Volume { volume: e.volume });
            }
        }
    }

    pub(crate) fn set_muted(&mut self, muted: bool) {
        let proxy = self.conn.with_proxy(
            "org.local.PipewireDBus",
            "/org/local/PipewireDBus",
            Duration::from_millis(5000),
        );

        if let Err(err) = proxy.set_muted(muted) {
            log::error!("failed to call SetMuted: {:?}", err);
        }
    }

    pub(crate) fn set_volume(&mut self, volume: f64) {
        let proxy = self.conn.with_proxy(
            "org.local.PipewireDBus",
            "/org/local/PipewireDBus",
            Duration::from_millis(5000),
        );

        if let Err(err) = proxy.set_volume(volume) {
            log::error!("failed to call SetVolume: {:?}", err);
        }
    }
}

impl AsRawFd for Pipewire {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        self.conn.channel().watch().fd
    }
}
