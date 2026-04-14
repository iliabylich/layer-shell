use crate::{
    Event,
    event_queue::EventQueue,
    sansio::{FileReader, FileReaderKind, Satisfy, Wants},
    user_data::ModuleId,
};
use anyhow::{Context as _, Result};
use parser::Parser;

mod parser;

pub(crate) struct Memory {
    reader: FileReader,
}

impl Memory {
    pub(crate) fn new() -> Self {
        Self {
            reader: FileReader::new(c"/proc/meminfo", FileReaderKind::Memory),
        }
    }

    pub(crate) const fn module_id(&self) -> ModuleId {
        ModuleId::Memory
    }

    pub(crate) fn wants(&mut self) -> Wants {
        self.reader.wants()
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<()> {
        let Some(buf) = self.reader.satisfy(satisfy, res) else {
            return Ok(());
        };
        let s = core::str::from_utf8(buf).context("decoding error")?;
        let (used, total) = Parser::parse(s).context("parse error")?;

        EventQueue::push_back(Event::Memory { used, total });
        Ok(())
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) {
        if let Err(err) = self.try_satisfy(satisfy, res) {
            log::error!("Memory module crashed: {satisfy:?} {res} {err:?}");
            self.reader.stop();
        }
    }

    pub(crate) fn tick(&mut self, _tick: u64) {
        self.reader.tick();
    }
}
