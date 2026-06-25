use crate::{
    Event,
    event_queue::EventQueue,
    sansio::{FileReader, Satisfy, Wants},
};
use anyhow::{Context as _, Result};
use parser::Parser;

mod parser;

pub(crate) struct Memory {
    reader: FileReader,
    buf: Box<[u8; 1_024]>,
}

impl Memory {
    pub(crate) fn new() -> Self {
        Self {
            reader: FileReader::new("/proc/meminfo"),
            buf: Box::new([0; _]),
        }
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        self.reader.wants(&mut *self.buf)
    }

    fn try_satisfy(&mut self, satisfy: Satisfy) -> Result<()> {
        let Some(buf) = self.reader.satisfy(satisfy, &*self.buf) else {
            return Ok(());
        };

        let (used, total) = Parser::parse(buf).context("parse error")?;
        EventQueue::push_back(Event::Memory { used, total });
        Ok(())
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy) {
        if let Err(err) = self.try_satisfy(satisfy) {
            log::error!("{err:?}");
            self.reader.stop();
        }
    }

    pub(crate) const fn tick(&mut self) {
        self.reader.tick();
    }
}
