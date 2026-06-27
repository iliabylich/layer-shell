use crate::{
    Event,
    actor::{CanStop, TryWantsTrySatisfy},
    event_queue::EventQueue,
    sansio::{FileReader, Satisfy, Wants},
    user_data::ModuleId,
};
use anyhow::Result;
use parser::{CoreUsage, Parser};

mod parser;

#[expect(clippy::upper_case_acronyms)]
pub(crate) enum CPU {
    Running {
        reader: FileReader,
        buf: Box<[u8; 1_024]>,
        state: Option<Vec<CoreUsage>>,
    },
    Stopped,
}

impl CPU {
    pub(crate) fn new() -> Self {
        Self::Running {
            reader: FileReader::new("/proc/stat"),
            buf: Box::new([0; _]),
            state: None,
        }
    }

    pub(crate) const fn tick(&mut self) {
        match self {
            CPU::Running { reader, .. } => reader.tick(),
            CPU::Stopped => todo!(),
        }
    }
}

impl TryWantsTrySatisfy for CPU {
    const ID: ModuleId = ModuleId::CPU;
    type Output = ();

    fn try_wants(&mut self) -> Result<Option<Wants>> {
        match self {
            CPU::Running { reader, buf, .. } => Ok(reader.wants(&mut **buf)),
            CPU::Stopped => Ok(None),
        }
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, events: &mut EventQueue) -> Result<Self::Output> {
        let Self::Running { reader, buf, state } = self else {
            return Ok(());
        };
        let Some(buf) = reader.try_satisfy(satisfy, &**buf)? else {
            return Ok(());
        };

        let prev = state.take();
        let next = Parser::parse(buf)?;

        let usage_per_core = diff(prev.as_deref(), &next)?.into();
        *state = Some(next);
        events.push_back(Event::CpuUsage { usage_per_core });
        Ok(())
    }
}

impl CanStop for CPU {
    fn stopped(&mut self) -> Self {
        Self::Stopped
    }
}

fn diff(prev: Option<&[CoreUsage]>, next: &[CoreUsage]) -> Result<Vec<u8>> {
    let Some(prev) = prev else {
        return Ok(vec![0; next.len()]);
    };

    prev.iter()
        .zip(next.iter())
        .map(|(prev, next)| next.load_comparing_to(*prev))
        .collect()
}
