use crate::{
    modules::FallibleModule,
    sansio::{Satisfy, Wants},
    user_data::ModuleId,
};
use anyhow::Result;
use mini_sansio_dbus::{DBusConnection, DBusQueue, IncomingMessage};

pub(crate) struct SystemDBus {
    conn: DBusConnection,
}

static mut READBUF: Vec<u8> = vec![];
fn readbuf() -> &'static mut Vec<u8> {
    unsafe { &mut READBUF }
}

static mut QUEUE: DBusQueue = DBusQueue::empty();
fn queue() -> &'static mut DBusQueue {
    unsafe { &mut QUEUE }
}

impl SystemDBus {
    pub(crate) fn init() {
        queue().push_hello();
    }

    pub(crate) fn new() -> Self {
        Self {
            conn: DBusConnection::new_system(),
        }
    }

    pub(crate) fn queue() -> &'static mut DBusQueue {
        queue()
    }
}

impl FallibleModule for SystemDBus {
    const MODULE_ID: ModuleId = ModuleId::SystemDBus;
    type Output = IncomingMessage<'static>;

    fn wants(&mut self) -> Option<Wants> {
        self.conn.wants(queue(), readbuf()).map(Wants::from)
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<Self::Output>> {
        self.conn
            .satisfy(satisfy.into(), res, readbuf(), queue())
            .map_err(Into::into)
    }
}
