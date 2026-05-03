use crate::{
    modules::Module,
    sansio::{Satisfy, Wants},
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

impl Module for SystemDBus {
    type Output = Option<IncomingMessage<'static>>;

    fn wants(&mut self) -> Result<Option<Wants>> {
        Ok(self.conn.wants(queue(), readbuf()).map(Wants::from))
    }

    fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Self::Output {
        let result = self.conn.satisfy(satisfy.into(), res, readbuf(), queue());

        match result {
            Ok(message) => message,
            Err(err) => {
                log::error!("SystemDBus has crashed: {err:?}");
                self.conn.stop();
                None
            }
        }
    }
}
