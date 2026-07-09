use crate::{
    sansio::{DBusState, Satisfy, Wants},
    utils::{dbus::queue::SessionDBusQueue, getenv},
};
use anyhow::{Context, Result};
use dbus::IncomingMessage;
use rustix::net::SocketAddrUnix;

#[derive(Debug, Clone, Copy)]
pub(crate) struct SessionDBus {
    state: DBusState,
}

impl SessionDBus {
    pub(crate) fn address() -> Result<&'static [u8]> {
        let address =
            getenv(c"DBUS_SESSION_BUS_ADDRESS").context("$DBUS_SESSION_BUS_ADDRESS is not set")?;
        let mut iter = address.split(|b| *b == b'=');
        let _prefix = iter.next().context("malformed $DBUS_SESSION_BUS_ADDRESS")?;
        let path = iter.next().context("malformed $DBUS_SESSION_BUS_ADDRESS")?;
        Ok(path)
    }

    pub(crate) const fn new() -> Self {
        Self {
            state: DBusState::CanSocket,
        }
    }

    pub(crate) fn wants(
        &mut self,
        readbuf: &mut [u8],
        queue: &SessionDBusQueue,
        addr: &SocketAddrUnix,
    ) -> Option<Wants> {
        self.state.wants(addr, readbuf, queue)
    }

    #[must_use]
    pub(crate) fn satisfy<'r>(
        &mut self,
        satisfy: Satisfy,
        readbuf: &'r [u8],
        queue: &mut SessionDBusQueue,
    ) -> Option<IncomingMessage<'r>> {
        let message;
        (self.state, message) = self.state.satisfy(satisfy, readbuf, queue);
        message
    }
}
