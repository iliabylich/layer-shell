use crate::{
    Event, UserData,
    liburing::IoUring,
    macros::report_and_exit,
    sansio::{FileReader, Satisfy, Wants},
    user_data::ModuleId,
};
use anyhow::{Context as _, Result};
use parser::Parser;
use store::Store;

mod parser;
mod store;

#[expect(clippy::upper_case_acronyms)]
pub(crate) struct CPU {
    reader: FileReader,
    store: Store,
}

impl CPU {
    pub(crate) const MODULE_ID: ModuleId = ModuleId::CPU;

    pub(crate) fn new() -> Self {
        Self {
            reader: FileReader::new(c"/proc/stat"),
            store: Store::new(),
        }
    }

    fn schedule_next(&mut self) {
        match self.reader.wants() {
            Wants::OpenAt {
                dfd,
                path,
                flags,
                mode,
            } => {
                let mut sqe = IoUring::get_sqe();
                sqe.prep_openat(dfd, path, flags, mode);
                sqe.set_user_data(UserData::new(Self::MODULE_ID, Satisfy::OpenAt));
            }
            Wants::Read { fd, buf, len } => {
                let mut sqe = IoUring::get_sqe();
                sqe.prep_read(fd, buf, len);
                sqe.set_user_data(UserData::new(Self::MODULE_ID, Satisfy::Read));
            }
            Wants::Nothing => {}

            _ => unreachable!(),
        }
    }

    pub(crate) fn init(&mut self) {
        self.schedule_next();
    }

    fn try_process(&mut self, satisfy: Satisfy, res: i32, events: &mut Vec<Event>) -> Result<()> {
        let Some(buf) = self.reader.satisfy(satisfy, res)? else {
            return Ok(());
        };
        let s = std::str::from_utf8(buf).context("decoding error")?;
        let data = Parser::parse_all(s).context("parse error")?;

        let usage_per_core = self.store.update(data);
        let event = Event::CpuUsage {
            usage_per_core: usage_per_core.into(),
        };
        events.push(event);
        Ok(())
    }

    pub(crate) fn process(&mut self, op: u8, res: i32, events: &mut Vec<Event>) {
        let satisfy = Satisfy::from(op);

        self.try_process(satisfy, res, events)
            .unwrap_or_else(|err| report_and_exit!("{err:?}"));

        self.schedule_next();
    }

    pub(crate) fn tick(&mut self) {
        self.reader.tick();
        self.schedule_next();
    }

    // fn schedule_open(&self) {
    //     let mut sqe = IoUring::get_sqe();
    //     sqe.prep_openat(AT_FDCWD, c"/proc/stat".as_ptr(), O_RDONLY, 0);
    //     sqe.set_user_data(UserData::new(ModuleId::CPU, Op::OpenAt));
    // }

    // fn schedule_read(&mut self) {
    //     let mut sqe = IoUring::get_sqe();
    //     sqe.prep_read(self.fd, self.buf.as_mut_ptr(), self.buf.len());
    //     sqe.set_user_data(UserData::new(ModuleId::CPU, Op::Read));
    // }

    // fn try_process(&mut self, op: Op, res: i32, events: &mut Vec<Event>) -> Result<()> {
    //     match op {
    //         Op::OpenAt => {
    //             ensure!(res > 0);
    //             self.fd = res;
    //             Ok(())
    //         }
    //         Op::Read => {
    //             ensure!(res >= 0);
    //             let len = res as usize;

    //             let s = std::str::from_utf8(&self.buf[..len]).context("decoding error")?;
    //             let data = Parser::parse_all(s).context("parse error")?;

    //             let usage_per_core = self.store.update(data);
    //             let event = Event::CpuUsage {
    //                 usage_per_core: usage_per_core.into(),
    //             };
    //             events.push(event);
    //             Ok(())
    //         }
    //     }
    // }

    // pub(crate) fn process(&mut self, op: u8, res: i32, events: &mut Vec<Event>) {
    //     if !self.healthy {
    //         return;
    //     }

    //     let op = Op::from(op);

    //     if let Err(err) = self.try_process(op, res, events) {
    //         log::error!("CPU::{op:?}({res} {err:?}");
    //         self.healthy = false;
    //     }
    // }

    // pub(crate) fn tick(&mut self) {
    //     if self.fd != -1 {
    //         self.schedule_read();
    //     }
    // }
}
