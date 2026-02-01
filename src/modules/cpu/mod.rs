use crate::{Event, UserData, liburing::IoUring, timerfd::Tick, user_data::ModuleId};
use anyhow::{Result, ensure};
use parser::Parser;
use std::{fs::File, os::fd::IntoRawFd};
use store::Store;

mod parser;
mod store;

#[expect(clippy::upper_case_acronyms)]
pub(crate) struct CPU {
    fd: i32,
    buf: [u8; 1_024],
    store: Store,
}

#[repr(u8)]
enum Op {
    Read,
}

impl CPU {
    pub(crate) fn new() -> Result<Box<Self>> {
        Ok(Box::new(Self {
            fd: File::open("/proc/stat")?.into_raw_fd(),
            buf: [0; 1_024],
            store: Store::new(),
        }))
    }

    pub(crate) fn feed(&mut self, op_id: u8, res: i32, events: &mut Vec<Event>) -> Result<()> {
        if op_id == Op::Read as u8 {
            ensure!(res > 0);
            let len = res as usize;
            let s = std::str::from_utf8(&self.buf[..len])?;
            let data = Parser::parse_all(s)?;

            let usage_per_core = self.store.update(data)?;
            let event = Event::CpuUsage {
                usage_per_core: usage_per_core.into(),
            };
            events.push(event);

            return Ok(());
        }

        Ok(())
    }

    pub(crate) fn tick(&mut self, tick: Tick, ring: &mut IoUring) -> Result<bool> {
        if tick.is_multiple_of(1) {
            let mut sqe = ring.get_sqe()?;
            sqe.prep_read(self.fd, self.buf.as_mut_ptr(), self.buf.len());
            sqe.set_user_data(UserData::new(ModuleId::CPU, Op::Read as u8));

            Ok(true)
        } else {
            Ok(false)
        }
    }
}
