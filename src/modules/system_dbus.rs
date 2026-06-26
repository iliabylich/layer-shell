use crate::{
    event_queue::EventQueue,
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

static mut READBUF: Vec<u8> = vec![];
fn readbuf() -> &'static mut Vec<u8> {
    unsafe { &mut READBUF }
}

static mut QUEUE: DBusQueue = DBusQueue::new();
const fn queue() -> &'static mut DBusQueue {
    unsafe { &mut QUEUE }
}

impl SystemDBus {
    pub(crate) fn init() -> Result<()> {
        queue().push_hello()?;
        readbuf().resize(400 * 1_024, 0);
        Ok(())
    }

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

    pub(crate) const fn queue() -> &'static mut DBusQueue {
        queue()
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        self.state.wants(&self.addr, readbuf(), queue())
    }

    pub(crate) fn satisfy(
        &mut self,
        satisfy: Satisfy,
        _events: &mut EventQueue,
    ) -> Option<IncomingMessage<'static>> {
        let message;
        (self.state, message) = self.state.satisfy(satisfy, readbuf(), queue());
        message
    }
}
