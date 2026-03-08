use crate::{
    Event, UserData,
    liburing::IoUring,
    macros::report_and_exit,
    sansio::{FileReader, Satisfy, Wants},
    user_data::ModuleId,
};
use anyhow::{Context as _, Result};
use parser::Parser;

mod parser;

pub(crate) struct Memory {
    reader: FileReader,
}

impl Memory {
    pub(crate) const MODULE_ID: ModuleId = ModuleId::Memory;

    pub(crate) fn new() -> Self {
        Self {
            reader: FileReader::new(c"/proc/meminfo"),
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
        let (used, total) = Parser::parse(s).context("parse error")?;

        events.push(Event::Memory { used, total });
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
}
