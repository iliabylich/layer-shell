use crate::{
    sansio::{DBusState, Satisfy, Wants},
    utils::{dbus::queue::SessionDBusQueue, getenv},
};
use anyhow::{Context, Result};
use dbus::IncomingMessage;
use rustix::net::{SocketAddrAny, SocketAddrUnix};

#[derive(Debug, Clone, Copy)]
pub(crate) struct SessionDBus {
    state: DBusState,
}

impl SessionDBus {
    pub(crate) fn address() -> Result<SocketAddrAny> {
        let address =
            getenv(c"DBUS_SESSION_BUS_ADDRESS").context("$DBUS_SESSION_BUS_ADDRESS is not set")?;
        let mut iter = address.split(|b| *b == b'=');
        let _prefix = iter.next().context("malformed $DBUS_SESSION_BUS_ADDRESS")?;
        let path = iter.next().context("malformed $DBUS_SESSION_BUS_ADDRESS")?;
        let addr = SocketAddrUnix::new(path).map_err(|errno| anyhow::anyhow!(errno))?;
        Ok(addr.into())
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
        addr: &SocketAddrAny,
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
