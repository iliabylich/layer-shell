use crate::{
    dbus::{Message, MessageDecoder},
    modules::Module,
    sansio::{DBusConnection, DBusQueue, Satisfy, Wants},
    unix_socket::new_unix_socket,
    user_data::ModuleId,
};
use anyhow::{Context, Result};

pub(crate) struct SystemDBus {
    conn: DBusConnection,
}

impl SystemDBus {
    pub(crate) fn new(queue: DBusQueue) -> Self {
        fn socket_path() -> String {
            std::env::var("DBUS_SYSTEM_BUS_ADDRESS")
                .context("no DBUS_SYSTEM_BUS_ADDRESS")
                .and_then(|address| {
                    address
                        .split_once("=")
                        .map(|(_, path)| path.to_string())
                        .context("malformed DBUS_SESSION_BUS_ADDRESS")
                })
                .unwrap_or_else(|_| String::from("/var/run/dbus/system_bus_socket"))
        }

        let addr = new_unix_socket(socket_path().as_bytes());

        Self {
            conn: DBusConnection::new(addr, queue),
        }
    }
}

impl Module for SystemDBus {
    type Output = Option<Message<'static>>;
    type Error = anyhow::Error;

    const MODULE_ID: ModuleId = ModuleId::SystemDBus;

    fn wants(&mut self) -> Wants {
        self.conn.wants()
    }

    fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Self::Output, Self::Error> {
        let Some(buf) = self.conn.satisfy(satisfy, res)? else {
            return Ok(None);
        };

        let message = MessageDecoder::decode(buf)?;
        Ok(Some(message))
    }

    fn tick(&mut self, _tick: u64) {}
}
