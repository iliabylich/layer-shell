use crate::{
    dbus::ConnectionKind,
    liburing::IoUring,
    macros::report_and_exit,
    user_data::{ModuleId, UserData},
};
use anyhow::{Context as _, Result};
use libc::{AF_UNIX, SOCK_STREAM, sockaddr, sockaddr_un};

pub(crate) struct Connector {
    module_id: ModuleId,
    addr: sockaddr_un,
    kind: ConnectionKind,
    fd: i32,
    healthy: bool,
}

#[repr(u8)]
#[derive(Debug)]
enum Op {
    Socket,
    Connect,
}
const MAX_OP: u8 = Op::Connect as u8;

impl From<u8> for Op {
    fn from(value: u8) -> Self {
        if value > MAX_OP {
            report_and_exit!("unsupported op in DBus connector: {value}")
        }
        unsafe { std::mem::transmute::<u8, Self>(value) }
    }
}

impl Connector {
    pub(crate) fn new(kind: ConnectionKind) -> Self {
        let module_id = match kind {
            ConnectionKind::Session => ModuleId::SessionDBusConnector,
            ConnectionKind::System => ModuleId::SystemDBusConnector,
        };

        Self {
            module_id,
            addr: unsafe { std::mem::zeroed() },
            kind,
            fd: -1,
            healthy: true,
        }
    }

    fn schedule_socket(&self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_socket(AF_UNIX, SOCK_STREAM, 0, 0);
        sqe.set_user_data(UserData::new(self.module_id, Op::Socket as u8));
    }

    fn schedule_connect(&mut self) {
        let path = match self.kind {
            ConnectionKind::Session => match session_socket_path() {
                Ok(ok) => ok,
                Err(err) => {
                    log::error!("failed to get session DBus socket path: {err:?}");
                    return;
                }
            },
            ConnectionKind::System => system_socket_path(),
        };

        self.addr = sockaddr_un {
            sun_family: AF_UNIX as u16,
            sun_path: {
                let path = unsafe { std::mem::transmute::<&[u8], &[i8]>(path.as_bytes()) };
                let mut out = [0; 108];
                out[..path.len()].copy_from_slice(path);
                out
            },
        };

        let mut sqe = IoUring::get_sqe();
        sqe.prep_connect(
            self.fd,
            (&self.addr as *const sockaddr_un).cast::<sockaddr>(),
            std::mem::size_of::<sockaddr_un>() as u32,
        );
        sqe.set_user_data(UserData::new(self.module_id, Op::Connect as u8));
    }

    pub(crate) fn init(&self) {
        self.schedule_socket();
    }

    pub(crate) fn process(&mut self, op: u8, res: i32) -> Option<i32> {
        if !self.healthy {
            return None;
        }

        let op = Op::from(op);

        macro_rules! crash {
            ($($arg:tt)*) => {{
                log::error!($($arg)*);
                self.healthy = false;
                return None;
            }};
        }

        match op {
            Op::Socket => {
                if res <= 0 {
                    crash!("{op:?}: res is {res}");
                }
                self.fd = res as i32;
                self.schedule_connect();
                None
            }
            Op::Connect => {
                if res < 0 {
                    crash!("{op:?}: res is {res}");
                }
                Some(self.fd)
            }
        }
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
