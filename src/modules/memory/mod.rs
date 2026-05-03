use crate::{
    Event,
    event_queue::EventQueue,
    modules::Module,
    sansio::{FileReader, FileReaderKind, Satisfy, Wants},
};
use anyhow::{Context as _, Result};
use parser::Parser;

mod parser;

pub(crate) struct Memory {
    reader: FileReader,
}

impl Memory {
    pub(crate) const fn new() -> Self {
        Self {
            reader: FileReader::new(c"/proc/meminfo", FileReaderKind::Memory),
        }
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

    pub(crate) const fn tick(&mut self, _tick: u64) {
        self.reader.tick();
    }
}

impl Module for Memory {
    type Output = ();

    fn wants(&mut self) -> Result<Option<Wants>> {
        self.reader.wants()
    }

    fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Self::Output {
        if let Err(err) = self.try_satisfy(satisfy, res) {
            log::error!("Memory module crashed: {satisfy:?} {res} {err:?}");
            self.reader.stop();
        }
    }
}
