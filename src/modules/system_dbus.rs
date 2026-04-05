use crate::{
    dbus::decoder::IncomingMessage,
    sansio::{DBusConnection, DBusConnectionKind, Satisfy, Wants},
    unix_socket::new_unix_socket,
    user_data::ModuleId,
};
use anyhow::Context;

pub(crate) struct SystemDBus {
    conn: DBusConnection,
}

impl SystemDBus {
    pub(crate) fn new() -> Self {
        fn socket_path() -> String {
            std::env::var("DBUS_SYSTEM_BUS_ADDRESS")
                .context("no DBUS_SYSTEM_BUS_ADDRESS")
                .and_then(|address| {
                    address
                        .split_once("=")
                        .map(|(_, path)| path.to_string())
                        .context("malformed DBUS_SYSTEM_BUS_ADDRESS")
                })
                .unwrap_or_else(|_| String::from("/var/run/dbus/system_bus_socket"))
        }

        let addr = new_unix_socket(socket_path().as_bytes());

        Self {
            conn: DBusConnection::new(addr, DBusConnectionKind::System),
        }
    }

    pub(crate) const fn module_id(&self) -> ModuleId {
        ModuleId::SystemDBus
    }

    pub(crate) fn wants(&mut self) -> Wants {
        self.conn.wants()
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Option<IncomingMessage<'_>> {
        let buf = self.conn.satisfy(satisfy, res)?;

        match IncomingMessage::new(buf) {
            Ok(message) => Some(message),
            Err(err) => {
                log::error!("DBus(system) got malformed message: {err:?}");
                self.conn.stop();
                None
            }
        }
    }
}
