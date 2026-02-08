use crate::{
    UserData,
    liburing::IoUring,
    macros::define_op,
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

define_op!("Hyprland Reader", Socket, Connect, Read);

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
        sqe.set_user_data(UserData::new(ModuleId::HyprlandReader, Op::Socket));
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
        sqe.set_user_data(UserData::new(ModuleId::HyprlandReader, Op::Connect));
    }

    fn schedule_read(&mut self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_read(self.fd, self.buf.as_mut_ptr(), self.buf.len());
        sqe.set_user_data(UserData::new(ModuleId::HyprlandReader, Op::Read));
    }

    pub(crate) fn init(&mut self) {
        self.schedule_socket()
    }

    pub(crate) fn process(&mut self, op: u8, res: i32, events: &mut Vec<HyprlandEvent>) {
        if !self.healthy {
            return;
        }

        let op = Op::from(op);

        macro_rules! assert_or_unhealthy {
            ($cond:expr, $($arg:tt)*) => {
                if !$cond {
                    log::error!("Hyprland::Reader::{op:?}");
                    log::error!($($arg)*);
                    self.healthy = false;
                    return;
                }
            };
        }

        match op {
            Op::Socket => {
                assert_or_unhealthy!(res > 0, "res = {res}");
                self.fd = res;
                self.schedule_connect();
            }
            Op::Connect => {
                assert_or_unhealthy!(res >= 0, "res = {res}");
                self.schedule_read();
            }
            Op::Read => {
                assert_or_unhealthy!(res > 0, "res = {res}");
                let len = res as usize;

                let s = std::str::from_utf8(&self.buf[..len]);
                assert_or_unhealthy!(s.is_ok(), "decoding error: {s:?}");
                let s = unsafe { s.unwrap_unchecked() };

                for line in s.lines() {
                    let event = HyprlandEvent::try_parse(line);
                    assert_or_unhealthy!(event.is_ok(), "parse error: {event:?}");
                    let event = unsafe { event.unwrap_unchecked() };

                    if let Some(event) = event {
                        events.push(event)
                    };
                }

                self.schedule_read();
            }
        }
    }
}
