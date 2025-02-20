use crate::{
    dbus::{OrgLocalPipewireDBus, OrgLocalPipewireDBusDataChanged},
    epoll::{FdId, Reader},
    modules::maybe_connected::MaybeConnected,
    Event, VerboseSender,
};
use anyhow::{Context as _, Result};
use dbus::{
    blocking::Connection,
    channel::{BusType, Channel},
    message::SignalArgs as _,
};
use std::time::Duration;

pub(crate) struct Pipewire {
    tx: VerboseSender<Event>,
    conn: Connection,
}

impl Pipewire {
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

    pub(crate) fn new(tx: VerboseSender<Event>) -> MaybeConnected<Self> {
        MaybeConnected::new(Self::try_new(tx))
    }
}

impl Reader for Pipewire {
    type Output = ();

    const NAME: &str = "Pipewire";

    fn read(&mut self) -> Result<Self::Output> {
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

    fn fd(&self) -> i32 {
        self.conn.channel().watch().fd
    }

    fn fd_id(&self) -> FdId {
        FdId::PipewireDBus
    }
}
