use crate::{
    Event,
    event_queue::EventQueue,
    modules::FallibleModule,
    sansio::{FileReader, Satisfy, Wants},
    user_data::ModuleId,
};
use anyhow::{Context as _, Result, bail};
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
    const MODULE_ID: ModuleId = ModuleId::Memory;
    type Output = ();

    fn wants(&mut self) -> Option<Wants> {
        self.reader.wants()
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<Self::Output>> {
        match satisfy {
            Satisfy::OpenAt => {
                self.reader.satisfy_open(res)?;
                Ok(None)
            }

            Satisfy::Read => {
                let buf = self.reader.satisfy_read(res)?;
                let (used, total) = Parser::parse(buf).context("parse error")?;
                EventQueue::push_back(Event::Memory { used, total });
                Ok(None)
            }

            _ => bail!("Memory module only suports OpenAt and Read"),
        }
    }

    fn try_tick(&mut self, _tick: u64) -> Result<()> {
        self.reader.satisfy_tick();
        Ok(())
    }
}
