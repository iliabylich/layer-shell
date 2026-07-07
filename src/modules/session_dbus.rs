use crate::{
    sansio::{DBusState, Satisfy, Wants},
    utils::{dbus::queue::SessionDBusQueue, getenv},
};
use anyhow::{Context, Result};
use dbus::IncomingMessage;
use rustix::net::SocketAddrUnix;

pub(crate) struct SessionDBus {
    state: DBusState,
    addr: SocketAddrUnix,
}

impl SessionDBus {
    fn try_new() -> Result<Self> {
        Ok(Self {
            state: DBusState::CanSocket,
            addr: SocketAddrUnix::new(address()?)?,
        })
    }

    pub(crate) fn new() -> Self {
        Self::try_new().unwrap_or_else(|err| {
            log::error!(target: "SessionDBus", "{err:?}");
            Self {
                state: DBusState::Disconnected,
                addr: unsafe { core::mem::zeroed() },
            }
        })
    }

    pub(crate) fn wants(&mut self, readbuf: &mut [u8], queue: &SessionDBusQueue) -> Option<Wants> {
        self.state.wants(&self.addr, readbuf, queue)
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

fn address() -> Result<&'static [u8]> {
    let address =
        getenv(c"DBUS_SESSION_BUS_ADDRESS").context("$DBUS_SESSION_BUS_ADDRESS is not set")?;
    let mut iter = address.split(|b| *b == b'=');
    let _prefix = iter.next().context("malformed $DBUS_SESSION_BUS_ADDRESS")?;
    let path = iter.next().context("malformed $DBUS_SESSION_BUS_ADDRESS")?;
    Ok(path)
}
