use crate::{
    UserData,
    liburing::IoUring,
    modules::hyprland::{event::HyprlandEvent, hyprland_instance_signature, xdg_runtime_dir},
    user_data::ModuleId,
};
use anyhow::{Result, ensure};
use std::os::{fd::IntoRawFd as _, unix::net::UnixStream};

#[derive(Debug)]
enum State {
    CanRead,
    Reading,
}

pub(crate) struct HyprlandReader {
    fd: i32,
    buf: [u8; 1_024],
    state: State,
}

#[repr(u8)]
enum Op {
    Read,
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
            state: State::CanRead,
        }))
    }

    pub(crate) fn drain(&mut self, ring: &mut IoUring) -> Result<bool> {
        match self.state {
            State::CanRead => {
                let mut sqe = ring.get_sqe()?;
                sqe.prep_read(self.fd, self.buf.as_mut_ptr(), self.buf.len());
                sqe.set_user_data(UserData::new(ModuleId::HyprlandReader, Op::Read as u8));

                self.state = State::Reading;
                Ok(true)
            }
            State::Reading => Ok(false),
        }
    }

    pub(crate) fn feed(
        &mut self,
        op_id: u8,
        res: i32,
        events: &mut Vec<HyprlandEvent>,
    ) -> Result<()> {
        if op_id == Op::Read as u8 {
            ensure!(
                matches!(self.state, State::Reading),
                "malformed state, expected Reading, got {:?}",
                self.state
            );

            ensure!(res > 0);
            let len = res as usize;
            let s = std::str::from_utf8(&self.buf[..len])?;
            for line in s.lines() {
                if let Some(event) = HyprlandEvent::try_parse(line)? {
                    events.push(event)
                };
            }

            self.state = State::CanRead
        }

        Ok(())
    }
}
