use crate::{
    dbus::{OrgLocalPipewireDBus, OrgLocalPipewireDBusDataChanged},
    Event, VerboseSender,
};
use anyhow::{Context as _, Result};
use dbus::{
    blocking::Connection,
    channel::{BusType, Channel},
    message::SignalArgs as _,
};
use std::time::Duration;

pub(crate) struct ConnectedPipewire {
    tx: VerboseSender<Event>,
    conn: Connection,
}

impl ConnectedPipewire {
    fn try_new(tx: VerboseSender<Event>) -> Result<Self> {
        let mut channel =
            Channel::get_private(BusType::Session).context("failed to connect to DBus")?;
        channel.set_watch_enabled(true);
        let conn = Connection::from(channel);

        let proxy = conn.with_proxy(
            "org.local.PipewireDBus",
            "/org/local/PipewireDBus",
            Duration::from_millis(1000),
        );
        let (volume, muted) = proxy.data().context("failed to get .Data")?;
        tx.send(Event::Volume { volume, muted });

        conn.add_match(
            OrgLocalPipewireDBusDataChanged::match_rule(None, None),
            |_: OrgLocalPipewireDBusDataChanged, _, _| true,
        )
        .context("failed to add_match")?;

        Ok(Self { tx, conn })
    }

    fn read(&mut self) {
        while let Ok(Some(message)) = self
            .conn
            .channel()
            .blocking_pop_message(Duration::from_secs(0))
        {
            if let Some(e) = OrgLocalPipewireDBusDataChanged::from_message(&message) {
                let event = Event::Volume {
                    volume: e.volume,
                    muted: e.muted,
                };
                self.tx.send(event);
            }
        }
    }

    fn fd(&self) -> i32 {
        self.conn.channel().watch().fd
    }
}

pub(crate) enum Pipewire {
    Connected(ConnectedPipewire),
    Disconnected,
}

impl Pipewire {
    pub(crate) fn new(tx: VerboseSender<Event>) -> Self {
        ConnectedPipewire::try_new(tx)
            .inspect_err(|err| log::error!("{:?}", err))
            .map(Self::Connected)
            .unwrap_or(Self::Disconnected)
    }

    pub(crate) fn read(&mut self) {
        if let Self::Connected(pipewire) = self {
            pipewire.read();
        }
    }

    pub(crate) fn fd(&self) -> Option<i32> {
        if let Self::Connected(pipewire) = self {
            Some(pipewire.fd())
        } else {
            None
        }
    }
}
