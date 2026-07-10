use crate::{
    sansio::{DBusState, Satisfy, Wants},
    utils::{dbus::queue::SystemDBusQueue, getenv, new_sockaddr_un},
};
use anyhow::{Context as _, Result};
use dbus::IncomingMessage;

#[derive(Debug, Clone, Copy)]
pub(crate) struct SystemDBus {
    state: DBusState,
}

impl SystemDBus {
    pub(crate) fn address() -> Result<libc::sockaddr_un> {
        let path = match getenv(c"DBUS_SYSTEM_BUS_ADDRESS") {
            Some(address) => {
                let mut iter = address.split(|b| *b == b'=');
                let _prefix = iter.next().context("malformed $DBUS_SYSTEM_BUS_ADDRESS")?;
                iter.next().context("malformed $DBUS_SYSTEM_BUS_ADDRESS")?
            }
            None => b"/var/run/dbus/system_bus_socket",
        };

        let addr = new_sockaddr_un(path)?;
        Ok(addr)
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
        addr: &libc::sockaddr_un,
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
