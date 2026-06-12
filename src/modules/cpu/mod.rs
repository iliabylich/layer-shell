use crate::{
    Event,
    event_queue::EventQueue,
    modules::FallibleModule,
    sansio::{FileReader, Satisfy, Wants},
    user_data::ModuleId,
};
use anyhow::{Context as _, Result, bail};
use parser::{CoreUsage, Parser};

mod parser;

#[expect(clippy::upper_case_acronyms)]
pub(crate) struct CPU {
    reader: FileReader<1_024>,
    state: Option<Vec<CoreUsage>>,
}

impl CPU {
    pub(crate) fn new() -> Self {
        Self {
            reader: FileReader::new("/proc/stat"),
            state: None,
        }
    }
}

impl FallibleModule for CPU {
    const MODULE_ID: ModuleId = ModuleId::CPU;
    type Output = ();

    fn wants(&mut self) -> Result<Option<Wants>> {
        Ok(self.reader.wants())
    }

    fn try_satisfy(&mut self, satisfy: Satisfy) -> Result<Option<Self::Output>> {
        match satisfy {
            Satisfy::OpenAt(res) => {
                let fd = res?;
                self.reader.satisfy_open(fd)?;
                Ok(None)
            }

            Satisfy::Read(res) => {
                let bytes_read = res?;
                let buf = self.reader.satisfy_read(bytes_read)?;

                let prev = self.state.take();
                let next = Parser::parse(buf).context("parse error")?;

                let usage_per_core = diff(prev.as_deref(), &next)?.into();
                self.state = Some(next);
                EventQueue::push_back(Event::CpuUsage { usage_per_core });

                Ok(None)
            }

            _ => bail!("CPU module only supports OpenAt and Read"),
        }
    }

    fn try_tick(&mut self, _tick: u64) -> Result<()> {
        self.reader.satisfy_tick();
        Ok(())
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
