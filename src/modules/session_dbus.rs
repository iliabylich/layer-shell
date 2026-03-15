use crate::{
    dbus::decoder::IncomingMessage,
    macros::report_and_exit,
    sansio::{DBusConnection, DBusQueue, Satisfy, Wants},
    unix_socket::new_unix_socket,
    user_data::ModuleId,
};
use anyhow::{Context, Result};

pub(crate) struct SessionDBus {
    conn: DBusConnection,
}

impl SessionDBus {
    pub(crate) fn new(queue: DBusQueue) -> Self {
        fn socket_path() -> Result<String> {
            let address = std::env::var("DBUS_SESSION_BUS_ADDRESS")?;
            let (_, path) = address
                .split_once("=")
                .context("malformed DBUS_SESSION_BUS_ADDRESS")?;
            Ok(path.to_string())
        }

        let socket_path = socket_path().unwrap_or_else(|err| report_and_exit!("{err:?}"));
        let addr = new_unix_socket(socket_path.as_bytes());

        Self {
            conn: DBusConnection::new(addr, queue),
        }
    }

    pub(crate) const fn module_id(&self) -> ModuleId {
        ModuleId::SessionDBus
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
