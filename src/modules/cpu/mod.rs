use crate::{
    Event,
    event_queue::EventQueue,
    modules::Module,
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
    events: EventQueue,
}

impl CPU {
    pub(crate) fn new(events: EventQueue) -> Self {
        Self {
            reader: FileReader::new(c"/proc/stat"),
            store: Store::new(),
            events,
        }
    }
}

impl Module for CPU {
    type Output = ();
    type Error = anyhow::Error;

    const MODULE_ID: ModuleId = ModuleId::CPU;

    fn wants(&mut self) -> Wants {
        self.reader.wants()
    }

    fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Self::Output, Self::Error> {
        let Some(buf) = self.reader.satisfy(satisfy, res)? else {
            return Ok(());
        };
        let s = std::str::from_utf8(buf).context("decoding error")?;
        let data = Parser::parse_all(s).context("parse error")?;

        let usage_per_core = self.store.update(data);
        let event = Event::CpuUsage {
            usage_per_core: usage_per_core.into(),
        };
        self.events.push_back(event);
        Ok(())
    }

    fn tick(&mut self, _tick: u64) {
        self.reader.tick();
    }
}
