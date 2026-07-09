use crate::{
    sansio::{DBusState, Satisfy, Wants},
    utils::{dbus::queue::SystemDBusQueue, getenv},
};
use anyhow::{Context as _, Result};
use dbus::IncomingMessage;
use rustix::net::SocketAddrUnix;

#[derive(Debug, Clone, Copy)]
pub(crate) struct SystemDBus {
    state: DBusState,
}

impl SystemDBus {
    pub(crate) fn address() -> Result<&'static [u8]> {
        let Some(address) = getenv(c"DBUS_SYSTEM_BUS_ADDRESS") else {
            return Ok(b"/var/run/dbus/system_bus_socket");
        };

        let mut iter = address.split(|b| *b == b'=');
        let _prefix = iter.next().context("malformed $DBUS_SYSTEM_BUS_ADDRESS")?;
        let path = iter.next().context("malformed $DBUS_SYSTEM_BUS_ADDRESS")?;
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
        queue: &SystemDBusQueue,
        addr: &SocketAddrUnix,
    ) -> Option<Wants> {
        self.state.wants(addr, readbuf, queue)
    }

    pub(crate) fn satisfy<'r>(
        &mut self,
        satisfy: Satisfy,
        readbuf: &'r [u8],
        queue: &mut SystemDBusQueue,
    ) -> Option<IncomingMessage<'r>> {
        let message;
        (self.state, message) = self.state.satisfy(satisfy, readbuf, queue);
        message
    }
}
