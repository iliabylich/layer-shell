use crate::{
    UserData,
    liburing::IoUring,
    macros::define_op,
    modules::hyprland::{event::HyprlandEvent, hyprland_instance_signature, xdg_runtime_dir},
    unix_socket::{new_unix_socket, zero_unix_socket},
    user_data::ModuleId,
};
use anyhow::{Context as _, Result, ensure};
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
            addr: zero_unix_socket(),
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

        self.addr = new_unix_socket(
            format!("{xdg_runtime_dir}/hypr/{hyprland_instance_signature}/.socket2.sock")
                .as_bytes(),
        );

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

    fn try_process(&mut self, op: Op, res: i32, events: &mut Vec<HyprlandEvent>) -> Result<()> {
        match op {
            Op::Socket => {
                ensure!(res > 0);
                self.fd = res;
                self.schedule_connect();
                Ok(())
            }
            Op::Connect => {
                ensure!(res >= 0);
                self.schedule_read();
                Ok(())
            }
            Op::Read => {
                ensure!(res > 0);
                let len = res as usize;

                let s = std::str::from_utf8(&self.buf[..len]).context("decoding error")?;
                for line in s.lines() {
                    if let Some(event) = HyprlandEvent::try_parse(line).context("parse error")? {
                        events.push(event)
                    };
                }

                self.schedule_read();
                Ok(())
            }
        }
    }

    pub(crate) fn process(&mut self, op: u8, res: i32, events: &mut Vec<HyprlandEvent>) {
        if !self.healthy {
            return;
        }

        let op = Op::from(op);

        if let Err(err) = self.try_process(op, res, events) {
            log::error!("Hyprland::Reader::{op:?}({res} {err:?}");
            self.healthy = false;
        }
    }
}
