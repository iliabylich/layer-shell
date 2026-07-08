use crate::{
    Event,
    actor::{CanStop, TryWantsTrySatisfy},
    event_queue::EventQueue,
    sansio::{FileReader, Satisfy, Wants},
    user_data::ModuleId,
};
use alloc::boxed::Box;
use anyhow::Result;
use parser::Parser;

mod parser;

pub(crate) enum Memory {
    Running {
        reader: FileReader,
        buf: Box<[u8; 1_024]>,
    },
    Stopped,
}

impl Memory {
    pub(crate) fn new() -> Self {
        Self::Running {
            reader: FileReader::new(c"/proc/meminfo"),
            buf: Box::new([0; _]),
        }
    }

    pub(crate) const fn tick(&mut self) {
        match self {
            Self::Running { reader, .. } => reader.tick(),
            Self::Stopped => todo!(),
        }
    }
}

impl TryWantsTrySatisfy for Memory {
    const ID: ModuleId = ModuleId::Memory;
    type Output = ();

    fn try_wants(&mut self) -> Result<Option<Wants>> {
        match self {
            Self::Running { reader, buf } => Ok(reader.wants(&mut **buf)),
            Self::Stopped => Ok(None),
        }
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, events: &mut EventQueue) -> Result<Self::Output> {
        let Self::Running { reader, buf } = self else {
            return Ok(());
        };
        let Some(buf) = reader.try_satisfy(satisfy, &**buf)? else {
            return Ok(());
        };

        let (used, total) = Parser::parse(buf)?;
        events.push_back(Event::Memory { used, total });
        Ok(())
    }
}

impl CanStop for Memory {
    fn stopped(&mut self) -> Self {
        Self::Stopped
    }
}
