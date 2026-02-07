use crate::{Event, UserData, liburing::IoUring, timerfd::Tick, user_data::ModuleId};
use anyhow::{Result, ensure};
use parser::Parser;
use std::{fs::File, os::fd::IntoRawFd};

mod parser;

pub(crate) struct Memory {
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

impl Memory {
    pub(crate) fn new() -> Result<Box<Self>> {
        Ok(Box::new(Self {
            fd: File::open("/proc/meminfo")?.into_raw_fd(),
            buf: [0; 1_024],
        }))
    }

    pub(crate) fn process(&mut self, op: u8, res: i32, events: &mut Vec<Event>) -> Result<()> {
        match Op::try_from(op)? {
            Op::Read => {
                ensure!(res > 0);
                let len = res as usize;
                let s = std::str::from_utf8(&self.buf[..len])?;

                let (used, total) = Parser::parse(s)?;
                events.push(Event::Memory { used, total });
            }
        }

        Ok(())
    }

    pub(crate) fn tick(&mut self, tick: Tick) -> Result<()> {
        if tick.is_multiple_of(1) {
            let mut sqe = IoUring::get_sqe()?;
            sqe.prep_read(self.fd, self.buf.as_mut_ptr(), self.buf.len());
            sqe.set_user_data(UserData::new(ModuleId::Memory, Op::Read as u8));
        }
        Ok(())
    }
}
