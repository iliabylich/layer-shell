use crate::{
    Event,
    event_queue::EventQueue,
    modules::FallibleModule,
    sansio::{FileReader, Satisfy, Wants},
};
use anyhow::{Context as _, Result};
use parser::Parser;

mod parser;

pub(crate) struct Memory {
    reader: FileReader<1_024>,
}

impl Memory {
    pub(crate) fn new() -> Self {
        Self {
            reader: FileReader::new(c"/proc/meminfo"),
        }
    }
}

impl FallibleModule for Memory {
    const NAME: &str = "Memory";
    type Output = ();

    fn try_wants(&mut self) -> Result<Option<Wants>> {
        Ok(self.reader.wants())
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<Self::Output>> {
        let Some(buf) = self.reader.try_satisfy(satisfy, res)? else {
            return Ok(None);
        };
        let (used, total) = Parser::parse(buf).context("parse error")?;

        EventQueue::push_back(Event::Memory { used, total });
        Ok(None)
    }

    fn try_tick(&mut self, _tick: u64) -> Result<()> {
        self.reader.tick();
        Ok(())
    }
}
