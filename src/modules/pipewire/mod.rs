use crate::{
    Event, VerboseSender,
    dbus::{OrgLocalPipewireDBus, OrgLocalPipewireDBusDataChanged},
    fd_id::FdId,
    modules::Module,
};
use anyhow::{Context as _, Result};
use dbus::{
    blocking::Connection,
    channel::{BusType, Channel},
    message::SignalArgs as _,
};
use std::{
    os::fd::{AsRawFd, RawFd},
    time::Duration,
};

pub(crate) struct Pipewire {
    tx: VerboseSender<Event>,
    conn: Connection,
}

impl Module for Pipewire {
    const FD_ID: FdId = FdId::PipewireDBus;
    const NAME: &str = "Pipewire";

    type ReadOutput = ();

    fn new(tx: VerboseSender<Event>) -> Result<Self> {
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

    fn read_events(&mut self) -> Result<()> {
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
        Ok(())
    }
}

impl AsRawFd for Pipewire {
    fn as_raw_fd(&self) -> RawFd {
        self.conn.channel().watch().fd
    }
}
