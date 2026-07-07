use crate::{
    sansio::{DBusState, Satisfy, Wants},
    utils::{dbus::queue::SystemDBusQueue, getenv},
};
use anyhow::{Context as _, Result};
use dbus::IncomingMessage;
use rustix::net::SocketAddrUnix;

pub(crate) struct SystemDBus {
    state: DBusState,
    addr: SocketAddrUnix,
}

impl SystemDBus {
    fn try_new() -> Result<Self> {
        Ok(Self {
            state: DBusState::CanSocket,
            addr: SocketAddrUnix::new(address()?)?,
        })
    }

    pub(crate) fn new() -> Self {
        Self::try_new().unwrap_or_else(|err| {
            log::error!(target: "SystemDBus", "{err:?}");
            Self {
                state: DBusState::Disconnected,
                addr: unsafe { core::mem::zeroed() },
            }
        })
    }

    pub(crate) fn wants(&mut self, readbuf: &mut [u8], queue: &SystemDBusQueue) -> Option<Wants> {
        self.state.wants(&self.addr, readbuf, queue)
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

fn address() -> Result<&'static [u8]> {
    let Some(address) = getenv(c"DBUS_SYSTEM_BUS_ADDRESS") else {
        return Ok(b"/var/run/dbus/system_bus_socket");
    };

    let mut iter = address.split(|b| *b == b'=');
    let _prefix = iter.next().context("malformed $DBUS_SYSTEM_BUS_ADDRESS")?;
    let path = iter.next().context("malformed $DBUS_SYSTEM_BUS_ADDRESS")?;
    Ok(path)
}
