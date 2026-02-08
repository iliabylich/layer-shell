use crate::{
    UserData,
    liburing::IoUring,
    modules::hyprland::{event::HyprlandEvent, hyprland_instance_signature, xdg_runtime_dir},
    user_data::ModuleId,
};
use libc::{AF_UNIX, SOCK_STREAM, sockaddr, sockaddr_un};

pub(crate) struct HyprlandReader {
    fd: i32,
    buf: [u8; 1_024],
    healthy: bool,
    addr: sockaddr_un,
}

#[repr(u8)]
#[derive(Debug)]
enum Op {
    Socket,
    Connect,
    Read,
}
const MAX_OP: u8 = Op::Read as u8;

impl From<u8> for Op {
    fn from(value: u8) -> Self {
        if value > MAX_OP {
            log::error!("unsupported op in HyprlandReaderOp: {value}");
            std::process::exit(1);
        }
        unsafe { std::mem::transmute::<u8, Self>(value) }
    }
}

impl HyprlandReader {
    pub(crate) fn new() -> Box<Self> {
        Box::new(Self {
            fd: -1,
            buf: [0; 1_024],
            healthy: true,
            addr: unsafe { std::mem::zeroed() },
        })
    }

    fn schedule_socket(&self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_socket(AF_UNIX, SOCK_STREAM, 0, 0);
        sqe.set_user_data(UserData::new(ModuleId::HyprlandReader, Op::Socket as u8));
    }

    fn schedule_connect(&mut self) {
        let Some(xdg_runtime_dir) = xdg_runtime_dir() else {
            self.healthy = false;
            return;
        };
        let Some(hyprland_instance_signature) = hyprland_instance_signature() else {
            self.healthy = false;
            return;
        };

        self.addr = sockaddr_un {
            sun_family: AF_UNIX as u16,
            sun_path: {
                let path =
                    format!("{xdg_runtime_dir}/hypr/{hyprland_instance_signature}/.socket2.sock");
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
        sqe.set_user_data(UserData::new(ModuleId::HyprlandReader, Op::Connect as u8));
    }

    fn schedule_read(&mut self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_read(self.fd, self.buf.as_mut_ptr(), self.buf.len());
        sqe.set_user_data(UserData::new(ModuleId::HyprlandReader, Op::Read as u8));
    }

    pub(crate) fn init(&mut self) {
        self.schedule_socket()
    }

    pub(crate) fn process(&mut self, op: u8, res: i32, events: &mut Vec<HyprlandEvent>) {
        if !self.healthy {
            return;
        }

        macro_rules! crash {
            ($($arg:tt)*) => {{
                log::error!($($arg)*);
                self.healthy = false;
                return;
            }};
        }

        let op = Op::from(op);

        match op {
            Op::Socket => {
                if res <= 0 {
                    crash!("{op:?}: res < 0: {res}");
                }
                self.fd = res as i32;
                self.schedule_connect();
            }
            Op::Connect => {
                if res < 0 {
                    crash!("{op:?}: res = {res}");
                }
                self.schedule_read();
            }
            Op::Read => {
                if res <= 0 {
                    crash!("{op:?}: res = {res}");
                }
                let len = res as usize;
                let s = match std::str::from_utf8(&self.buf[..len]) {
                    Ok(ok) => ok,
                    Err(err) => crash!("{op:?}: {err:?}"),
                };
                for line in s.lines() {
                    let event = match HyprlandEvent::try_parse(line) {
                        Ok(ok) => ok,
                        Err(err) => crash!("{err:?}"),
                    };
                    if let Some(event) = event {
                        events.push(event)
                    };
                }

                self.schedule_read();
            }
        }
    }
}
