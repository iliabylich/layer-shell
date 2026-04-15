use crate::{
    sansio::{Satisfy, Wants},
    user_data::ModuleId,
};
use mini_sansio_dbus::{DBusConnection, DBusQueue, IncomingMessage};

pub(crate) struct SystemDBus {
    conn: DBusConnection,
}

static mut READBUF: Vec<u8> = vec![];
fn readbuf() -> &'static mut Vec<u8> {
    unsafe { &mut READBUF }
}

static mut QUEUE: Option<DBusQueue> = None;
fn queue() -> &'static mut DBusQueue {
    unsafe { QUEUE.as_mut().unwrap() }
}

impl SystemDBus {
    pub(crate) fn new() -> Self {
        unsafe { QUEUE = Some(DBusQueue::new()) }

        Self {
            conn: DBusConnection::new_system(),
        }
    }

    pub(crate) fn queue() -> &'static mut DBusQueue {
        queue()
    }

    pub(crate) const fn module_id(&self) -> ModuleId {
        ModuleId::SystemDBus
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        self.conn.wants(queue(), readbuf()).map(Wants::from)
    }

    pub(crate) fn satisfy(
        &mut self,
        satisfy: Satisfy,
        res: i32,
    ) -> Option<IncomingMessage<'static>> {
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
