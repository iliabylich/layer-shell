use crate::{
    Event,
    modules::Module,
    sansio::{FileReader, Satisfy, Wants},
    user_data::ModuleId,
};
use anyhow::{Context as _, Result};
use parser::Parser;

mod parser;

pub(crate) struct Memory {
    reader: FileReader,
}

impl Module for Memory {
    type Input = ();
    type Output = ();
    type Error = anyhow::Error;

    const MODULE_ID: ModuleId = ModuleId::Memory;

    fn new((): Self::Input) -> Self {
        Self {
            reader: FileReader::new(c"/proc/meminfo"),
        }
    }

    fn wants(&mut self) -> Wants {
        self.reader.wants()
    }

    fn satisfy(
        &mut self,
        satisfy: Satisfy,
        res: i32,
        events: &mut Vec<Event>,
    ) -> Result<Option<Self::Output>, Self::Error> {
        let Some(buf) = self.reader.satisfy(satisfy, res)? else {
            return Ok(None);
        };
        let s = std::str::from_utf8(buf).context("decoding error")?;
        let (used, total) = Parser::parse(s).context("parse error")?;

        events.push(Event::Memory { used, total });
        Ok(None)
    }

    fn tick(&mut self, _tick: u64) {
        self.reader.tick();
    }
}
