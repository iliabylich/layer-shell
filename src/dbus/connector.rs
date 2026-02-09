use crate::{
    dbus::ConnectionKind,
    liburing::IoUring,
    macros::define_op,
    unix_socket::{new_unix_socket, zero_unix_socket},
    user_data::{ModuleId, UserData},
};
use anyhow::{Context as _, Result, ensure};
use libc::{AF_UNIX, SOCK_STREAM, sockaddr, sockaddr_un};

pub(crate) struct Connector {
    module_id: ModuleId,
    addr: sockaddr_un,
    kind: ConnectionKind,
    fd: i32,
    healthy: bool,
}

define_op!("DBus Connector", Socket, Connect);

impl Connector {
    pub(crate) fn new(kind: ConnectionKind) -> Self {
        let module_id = match kind {
            ConnectionKind::Session => ModuleId::SessionDBusConnector,
            ConnectionKind::System => ModuleId::SystemDBusConnector,
        };

        Self {
            module_id,
            addr: zero_unix_socket(),
            kind,
            fd: -1,
            healthy: true,
        }
    }

    fn schedule_socket(&self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_socket(AF_UNIX, SOCK_STREAM, 0, 0);
        sqe.set_user_data(UserData::new(self.module_id, Op::Socket));
    }

    fn schedule_connect(&mut self) {
        let Some(path) = socket_path(self.kind) else {
            self.healthy = false;
            return;
        };

        self.addr = new_unix_socket(path.as_bytes());

        let mut sqe = IoUring::get_sqe();
        sqe.prep_connect(
            self.fd,
            (&self.addr as *const sockaddr_un).cast::<sockaddr>(),
            std::mem::size_of::<sockaddr_un>() as u32,
        );
        sqe.set_user_data(UserData::new(self.module_id, Op::Connect));
    }

    pub(crate) fn init(&self) {
        self.schedule_socket();
    }

    fn try_process(&mut self, op: Op, res: i32) -> Result<Option<i32>> {
        match op {
            Op::Socket => {
                ensure!(res > 0);
                self.fd = res;
                self.schedule_connect();
                Ok(None)
            }
            Op::Connect => {
                ensure!(res >= 0);
                Ok(Some(self.fd))
            }
        }
    }

    pub(crate) fn process(&mut self, op: u8, res: i32) -> Option<i32> {
        if !self.healthy {
            return None;
        }

        let op = Op::from(op);

        match self.try_process(op, res) {
            Ok(ok) => ok,
            Err(err) => {
                log::error!("DBusConnector({:?})::{op:?}({res} {err:?}", self.kind);
                self.healthy = false;
                None
            }
        }
    }
}

fn socket_path(kind: ConnectionKind) -> Option<String> {
    match kind {
        ConnectionKind::Session => match session_socket_path() {
            Ok(ok) => Some(ok),
            Err(err) => {
                log::error!("failed to get session DBus socket path: {err:?}");
                None
            }
        },
        ConnectionKind::System => Some(system_socket_path()),
    }
}

fn session_socket_path() -> Result<String> {
    let address = std::env::var("DBUS_SESSION_BUS_ADDRESS")?;
    let (_, path) = address
        .split_once("=")
        .context("malformed DBUS_SESSION_BUS_ADDRESS")?;
    Ok(path.to_string())
}

fn system_socket_path() -> String {
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
