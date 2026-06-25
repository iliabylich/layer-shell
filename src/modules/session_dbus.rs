use crate::{
    sansio::{DBusState, Satisfy, Wants},
    utils::dbus::queue::DBusQueue,
};
use anyhow::{Context, Result};
use dbus::IncomingMessage;
use rustix::net::SocketAddrUnix;

pub(crate) struct SessionDBus {
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

impl SessionDBus {
    pub(crate) fn init() -> Result<()> {
        queue().push_hello()?;
        readbuf().resize(400 * 1_024, 0);
        Ok(())
    }

    fn try_new() -> Result<Self> {
        let address = std::env::var("DBUS_SESSION_BUS_ADDRESS")?;
        let (_, path) = address
            .split_once('=')
            .context("malformed $DBUS_SESSION_BUS_ADDRESS")?;

        Ok(Self {
            state: DBusState::CanSocket,
            addr: SocketAddrUnix::new(path)?,
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

    pub(crate) const fn queue() -> &'static mut DBusQueue {
        queue()
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        self.state.wants(&self.addr, readbuf(), queue())
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy) -> Option<IncomingMessage<'static>> {
        let message;
        (self.state, message) = self.state.satisfy(satisfy, readbuf(), queue());
        message
    }
}
