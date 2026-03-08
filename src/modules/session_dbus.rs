use crate::{
    dbus::{Message, MessageDecoder},
    liburing::IoUring,
    macros::report_and_exit,
    modules::DBusQueued,
    sansio::{DBusConnection, Satisfy, Wants},
    unix_socket::new_unix_socket,
    user_data::{ModuleId, UserData},
};
use anyhow::{Context, Result};

pub(crate) struct SessionDBus {
    conn: DBusConnection,
}

impl SessionDBus {
    pub(crate) fn new() -> Self {
        let socket_path = socket_path().unwrap_or_else(|err| report_and_exit!("{err:?}"));
        let addr = new_unix_socket(socket_path.as_bytes());

        Self {
            conn: DBusConnection::new(addr),
        }
    }

    pub(crate) fn module_id(&self) -> ModuleId {
        ModuleId::SessionDBus
    }

    pub(crate) fn wants(&mut self) -> Wants {
        self.conn.wants()
    }

    fn schedule_next_wanted(&mut self) {
        match self.wants() {
            Wants::Socket { domain, r#type } => {
                let mut sqe = IoUring::get_sqe();
                sqe.prep_socket(domain, r#type, 0, 0);
                sqe.set_user_data(UserData::new(self.module_id(), Satisfy::Socket));
            }
            Wants::Connect { fd, addr, addrlen } => {
                let mut sqe = IoUring::get_sqe();
                sqe.prep_connect(fd, addr, addrlen);
                sqe.set_user_data(UserData::new(self.module_id(), Satisfy::Connect));
            }
            Wants::Read { fd, buf, len } => {
                let mut sqe = IoUring::get_sqe();
                sqe.prep_read(fd, buf, len);
                sqe.set_user_data(UserData::new(self.module_id(), Satisfy::Read));
            }
            Wants::Write { fd, buf, len } => {
                let mut sqe = IoUring::get_sqe();
                sqe.prep_write(fd, buf, len);
                sqe.set_user_data(UserData::new(self.module_id(), Satisfy::Write));
            }
            Wants::ReadWrite {
                fd,
                readbuf,
                readlen,
                writebuf,
                writelen,
            } => {
                let mut sqe = IoUring::get_sqe();
                sqe.prep_read(fd, readbuf, readlen);
                sqe.set_user_data(UserData::new(self.module_id(), Satisfy::Read));

                let mut sqe = IoUring::get_sqe();
                sqe.prep_write(fd, writebuf, writelen);
                sqe.set_user_data(UserData::new(self.module_id(), Satisfy::Write));
            }
            Wants::Close { fd } => {
                let mut sqe = IoUring::get_sqe();
                sqe.prep_close(fd);
                sqe.set_user_data(UserData::new(self.module_id(), Satisfy::Close));
            }
            Wants::Nothing => {}
            Wants::OpenAt { .. } => unreachable!(),
        }
    }

    pub(crate) fn init(&mut self) {
        self.schedule_next_wanted();
    }

    fn try_process(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<Message<'static>>> {
        if let Some(buf) = self.conn.satisfy(satisfy, res)? {
            let message = MessageDecoder::decode(buf)?;
            Ok(Some(message))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn process(&mut self, op: u8, res: i32) -> Option<Message<'static>> {
        let satisfy = Satisfy::from(op);

        match self.try_process(satisfy, res) {
            Ok(message) => {
                self.schedule_next_wanted();
                message
            }
            Err(err) => report_and_exit!("{err:?}"),
        }
    }
}

fn socket_path() -> Result<String> {
    let address = std::env::var("DBUS_SESSION_BUS_ADDRESS")?;
    let (_, path) = address
        .split_once("=")
        .context("malformed DBUS_SESSION_BUS_ADDRESS")?;
    Ok(path.to_string())
}

impl DBusQueued for SessionDBus {
    fn enqueue(&mut self, message: &mut Message) {
        self.conn.enqueue(message);
        self.schedule_next_wanted();
    }
}
