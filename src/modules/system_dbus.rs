use crate::{
    modules::FallibleModule,
    sansio::{DBusState, Satisfy, Wants},
    unix_socket::new_unix_socket,
    user_data::ModuleId,
    utils::dbus::queue::DBusQueue,
};
use anyhow::Result;
use dbus::IncomingMessage;
use libc::sockaddr_un;

pub(crate) struct SystemDBus {
    state: DBusState,
    address: sockaddr_un,
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
        let address = std::env::var("DBUS_SYSTEM_BUS_ADDRESS")
            .ok()
            .and_then(|address| address.split_once('=').map(|(_, path)| path.to_string()))
            .unwrap_or_else(|| String::from("/var/run/dbus/system_bus_socket"));

        Ok(Self {
            state: DBusState::WantsSocket,
            address: new_unix_socket(address.as_bytes())?,
        })
    }

    pub(crate) fn new() -> Self {
        Self::try_new().unwrap_or_else(|err| {
            log::error!(target: Self::MODULE_ID.as_str(), "{err:?}");
            Self {
                state: DBusState::Disconnected,
                address: unsafe { core::mem::zeroed() },
            }
        })
    }

    pub(crate) const fn queue() -> &'static mut DBusQueue {
        queue()
    }
}

impl FallibleModule for SystemDBus {
    const MODULE_ID: ModuleId = ModuleId::SystemDBus;
    type Output = IncomingMessage<'static>;

    fn wants(&mut self) -> Result<Option<Wants>> {
        self.state.wants(&self.address, readbuf(), queue())
    }

    fn try_satisfy(&mut self, satisfy: Satisfy) -> Result<Option<Self::Output>> {
        let mut state = DBusState::Disconnected;
        std::mem::swap(&mut self.state, &mut state);
        let (state, message) = state.satisfy(satisfy, readbuf(), queue())?;
        self.state = state;
        Ok(message)
    }
}
