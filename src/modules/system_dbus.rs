use crate::{
    sansio::{DBusState, Satisfy, Wants},
    utils::dbus::queue::DBusQueue,
};
use anyhow::Result;
use dbus::IncomingMessage;
use rustix::net::SocketAddrUnix;

pub(crate) struct SystemDBus {
    state: DBusState,
    addr: SocketAddrUnix,
}

impl SystemDBus {
    fn try_new() -> Result<Self> {
        let path = std::env::var("DBUS_SYSTEM_BUS_ADDRESS")
            .ok()
            .and_then(|address| address.split_once('=').map(|(_, path)| path.to_string()))
            .unwrap_or_else(|| String::from("/var/run/dbus/system_bus_socket"));

        Ok(Self {
            state: DBusState::CanSocket,
            addr: SocketAddrUnix::new(path)?,
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

    pub(crate) fn wants(&mut self, readbuf: &mut Vec<u8>, queue: &DBusQueue) -> Option<Wants> {
        self.state.wants(&self.addr, readbuf, queue)
    }

    pub(crate) fn satisfy<'r>(
        &mut self,
        satisfy: Satisfy,
        readbuf: &'r [u8],
        queue: &mut DBusQueue,
    ) -> Option<IncomingMessage<'r>> {
        let message;
        (self.state, message) = self.state.satisfy(satisfy, readbuf, queue);
        message
    }
}
