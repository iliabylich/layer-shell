use crate::{
    dbus::decoder::IncomingMessage,
    sansio::{DBusConnection, DBusConnectionKind, Satisfy, Wants},
    unix_socket::new_unix_socket,
    user_data::ModuleId,
};
use anyhow::{Context, Result};

pub(crate) struct SessionDBus {
    conn: DBusConnection,
}

impl SessionDBus {
    pub(crate) fn new() -> Self {
        fn socket_path() -> Result<String> {
            let address = std::env::var("DBUS_SESSION_BUS_ADDRESS")?;
            let (_, path) = address
                .split_once("=")
                .context("malformed DBUS_SESSION_BUS_ADDRESS")?;
            Ok(path.to_string())
        }

        let socket_path = match socket_path() {
            Ok(path) => path,
            Err(err) => {
                log::error!("Failed to connect to session DBus: {err:?}");
                return Self {
                    conn: DBusConnection::dummy(DBusConnectionKind::Session),
                };
            }
        };
        let addr = new_unix_socket(socket_path.as_bytes());

        Self {
            conn: DBusConnection::new(addr, DBusConnectionKind::Session),
        }
    }

    pub(crate) const fn module_id(&self) -> ModuleId {
        ModuleId::SessionDBus
    }

    pub(crate) fn wants(&mut self) -> Wants {
        self.conn.wants()
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Option<IncomingMessage<'_>> {
        let buf = self.conn.satisfy(satisfy, res)?;

        match IncomingMessage::new(buf) {
            Ok(message) => Some(message),
            Err(err) => {
                log::error!("DBus(session) got malformed message: {err:?}");
                self.conn.stop();
                None
            }
        }
    }
}
