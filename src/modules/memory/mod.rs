use crate::{
    Event,
    event_queue::EventQueue,
    modules::Module,
    sansio::{FileReader, Satisfy, Wants},
    user_data::ModuleId,
};
use anyhow::{Context as _, Result};
use parser::Parser;

mod parser;

pub(crate) struct Memory {
    reader: FileReader,
    events: EventQueue,
}

impl Memory {
    pub(crate) fn new(events: EventQueue) -> Self {
        Self {
            reader: FileReader::new(c"/proc/meminfo"),
            events,
        }
    }
}

impl Module for Memory {
    type Output = ();
    type Error = anyhow::Error;

    const MODULE_ID: ModuleId = ModuleId::Memory;

    fn wants(&mut self) -> Wants {
        self.reader.wants()
    }

    fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Self::Output, Self::Error> {
        let Some(buf) = self.reader.satisfy(satisfy, res)? else {
            return Ok(());
        };
        let s = std::str::from_utf8(buf).context("decoding error")?;
        let (used, total) = Parser::parse(s).context("parse error")?;

        self.events.push_back(Event::Memory { used, total });
        Ok(())
    }

    fn tick(&mut self, _tick: u64) {
        self.reader.tick();
    }
}
