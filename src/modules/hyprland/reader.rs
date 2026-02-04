use crate::{
    UserData,
    liburing::IoUring,
    modules::hyprland::{event::HyprlandEvent, hyprland_instance_signature, xdg_runtime_dir},
    user_data::ModuleId,
};
use anyhow::{Result, ensure};
use std::os::{fd::IntoRawFd as _, unix::net::UnixStream};

pub(crate) struct HyprlandReader {
    fd: i32,
    buf: [u8; 1_024],
}

#[repr(u8)]
enum Op {
    Read,
}
const MAX_OP: u8 = Op::Read as u8;

impl TryFrom<u8> for Op {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self> {
        ensure!(value <= MAX_OP);
        unsafe { Ok(std::mem::transmute::<u8, Self>(value)) }
    }
}

impl HyprlandReader {
    pub(crate) fn new() -> Result<Box<Self>> {
        let path = format!(
            "{}/hypr/{}/.socket2.sock",
            xdg_runtime_dir()?,
            hyprland_instance_signature()?
        );

        let fd = UnixStream::connect(&path)?.into_raw_fd();

        Ok(Box::new(Self {
            fd,
            buf: [0; 1_024],
        }))
    }

    pub(crate) fn init(&mut self, ring: &mut IoUring) -> Result<()> {
        self.schedule_read(ring)
    }

    fn schedule_read(&mut self, ring: &mut IoUring) -> Result<()> {
        let mut sqe = ring.get_sqe()?;
        sqe.prep_read(self.fd, self.buf.as_mut_ptr(), self.buf.len());
        sqe.set_user_data(UserData::new(ModuleId::HyprlandReader, Op::Read as u8));
        Ok(())
    }

    pub(crate) fn process(
        &mut self,
        op: u8,
        res: i32,
        ring: &mut IoUring,
        events: &mut Vec<HyprlandEvent>,
    ) -> Result<()> {
        match Op::try_from(op)? {
            Op::Read => {
                ensure!(res > 0);
                let len = res as usize;
                let s = std::str::from_utf8(&self.buf[..len])?;
                for line in s.lines() {
                    if let Some(event) = HyprlandEvent::try_parse(line)? {
                        events.push(event)
                    };
                }

                self.schedule_read(ring)?;
            }
        }
        Ok(())
    }
}
