use crate::{Event, UserData, liburing::IoUring, macros::define_op, user_data::ModuleId};
use anyhow::{Context as _, Result, ensure};
use libc::{AT_FDCWD, O_RDONLY};
use parser::Parser;

mod parser;

pub(crate) struct Memory {
    fd: i32,
    buf: [u8; 1_024],
    healthy: bool,
}

define_op!("Memory", OpenAt, Read);

impl Memory {
    pub(crate) fn new() -> Box<Self> {
        Box::new(Self {
            fd: -1,
            buf: [0; 1_024],
            healthy: true,
        })
    }

    fn schedule_open(&self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_openat(AT_FDCWD, c"/proc/meminfo".as_ptr(), O_RDONLY, 0);
        sqe.set_user_data(UserData::new(ModuleId::Memory, Op::OpenAt));
    }

    fn schedule_read(&mut self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_read(self.fd, self.buf.as_mut_ptr(), self.buf.len());
        sqe.set_user_data(UserData::new(ModuleId::Memory, Op::Read));
    }

    pub(crate) fn init(&self) {
        self.schedule_open();
    }

    fn try_process(&mut self, op: Op, res: i32, events: &mut Vec<Event>) -> Result<()> {
        match op {
            Op::OpenAt => {
                ensure!(res > 0);
                self.fd = res;
                Ok(())
            }

            Op::Read => {
                ensure!(res > 0);
                let len = res as usize;

                let s = std::str::from_utf8(&self.buf[..len]).context("decoding error")?;
                let (used, total) = Parser::parse(s).context("parse error")?;

                events.push(Event::Memory { used, total });
                Ok(())
            }
        }
    }

    pub(crate) fn process(&mut self, op: u8, res: i32, events: &mut Vec<Event>) {
        if !self.healthy {
            return;
        }

        let op = Op::from(op);

        if let Err(err) = self.try_process(op, res, events) {
            log::error!("Memory::{op:?}({res} {err:?}");
            self.healthy = false;
        }
    }

    pub(crate) fn tick(&mut self) {
        if self.fd != -1 {
            self.schedule_read();
        }
    }
}
