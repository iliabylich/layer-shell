use crate::{
    Event,
    event_queue::EventQueue,
    modules::FallibleModule,
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
}

impl FallibleModule for Memory {
    const NAME: &str = "Memory";
    type Output = ();

    fn try_wants(&mut self) -> Result<Option<Wants>> {
        self.reader.wants()
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<Self::Output>> {
        let Some(buf) = self.reader.satisfy(satisfy, res) else {
            return Ok(None);
        };
        let s = core::str::from_utf8(buf).context("decoding error")?;
        let (used, total) = Parser::parse(s).context("parse error")?;

        EventQueue::push_back(Event::Memory { used, total });
        Ok(None)
    }

    fn try_tick(&mut self, _tick: u64) -> Result<()> {
        self.reader.tick();
        Ok(())
    }
}
