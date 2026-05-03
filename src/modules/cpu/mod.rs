use crate::{
    Event,
    event_queue::EventQueue,
    modules::FallibleModule,
    sansio::{FileReader, FileReaderKind, Satisfy, Wants},
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
    pub(crate) const fn new() -> Self {
        Self {
            reader: FileReader::new(c"/proc/stat", FileReaderKind::CPU),
            store: Store::new(),
        }
    }
}

impl FallibleModule for CPU {
    const NAME: &str = "CPU";
    type Output = ();

    fn try_wants(&mut self) -> Result<Option<Wants>> {
        self.reader.wants()
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<Self::Output>> {
        let Some(buf) = self.reader.satisfy(satisfy, res) else {
            return Ok(None);
        };
        let s = core::str::from_utf8(buf).context("decoding error")?;
        let data = Parser::parse_all(s).context("parse error")?;

        let usage_per_core = self.store.update(data)?;
        let event = Event::CpuUsage {
            usage_per_core: usage_per_core.into(),
        };
        EventQueue::push_back(event);
        Ok(None)
    }

    fn try_tick(&mut self, _tick: u64) -> Result<()> {
        self.reader.tick();
        Ok(())
    }
}
