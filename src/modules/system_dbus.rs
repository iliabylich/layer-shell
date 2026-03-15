use crate::{
    dbus::decoder::IncomingMessage,
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

    pub(crate) const fn module_id(&self) -> ModuleId {
        ModuleId::SystemDBus
    }

    pub(crate) fn wants(&mut self) -> Wants {
        self.conn.wants()
    }

    pub(crate) fn satisfy(
        &mut self,
        satisfy: Satisfy,
        res: i32,
    ) -> Result<Option<IncomingMessage<'_>>> {
        let Some(buf) = self.conn.satisfy(satisfy, res)? else {
            return Ok(None);
        };

        let message = IncomingMessage::new(buf)?;
        Ok(Some(message))
    }
}
