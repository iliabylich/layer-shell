use crate::{
    Event,
    event_queue::EventQueue,
    sansio::{FileReader, FileReaderKind, Satisfy, Wants},
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
    pub(crate) fn new() -> Self {
        Self {
            reader: FileReader::new(c"/proc/stat", FileReaderKind::CPU),
            store: Store::new(),
        }
    }

    pub(crate) const fn module_id(&self) -> ModuleId {
        ModuleId::CPU
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        self.reader.wants()
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<()> {
        let Some(buf) = self.reader.satisfy(satisfy, res) else {
            return Ok(());
        };
        let s = core::str::from_utf8(buf).context("decoding error")?;
        let data = Parser::parse_all(s).context("parse error")?;

        let usage_per_core = self.store.update(data);
        let event = Event::CpuUsage {
            usage_per_core: usage_per_core.into(),
        };
        EventQueue::push_back(event);
        Ok(())
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) {
        if let Err(err) = self.try_satisfy(satisfy, res) {
            log::error!("CPU module crashed: {satisfy:?} {res} {err:?}");
            self.reader.stop();
        }
    }

    pub(crate) fn tick(&mut self, _tick: u64) {
        self.reader.tick();
    }
}
