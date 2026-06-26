use crate::{
    Event,
    event_queue::EventQueue,
    sansio::{FileReader, Satisfy, Wants},
};
use anyhow::{Context as _, Result};
use parser::{CoreUsage, Parser};

mod parser;

#[expect(clippy::upper_case_acronyms)]
pub(crate) struct CPU {
    reader: FileReader,
    buf: Box<[u8; 1_024]>,
    state: Option<Vec<CoreUsage>>,
}

impl CPU {
    pub(crate) fn new() -> Self {
        Self {
            reader: FileReader::new("/proc/stat"),
            buf: Box::new([0; _]),
            state: None,
        }
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        self.reader.wants(&mut *self.buf)
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, events: &mut EventQueue) -> Result<()> {
        let Some(buf) = self.reader.satisfy(satisfy, &*self.buf) else {
            return Ok(());
        };

        let prev = self.state.take();
        let next = Parser::parse(buf).context("parse error")?;

        let usage_per_core = diff(prev.as_deref(), &next)?.into();
        self.state = Some(next);
        events.push_back(Event::CpuUsage { usage_per_core });
        Ok(())
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, events: &mut EventQueue) {
        if let Err(err) = self.try_satisfy(satisfy, events) {
            log::error!("{err:?}");
            self.reader.stop();
        }
    }

    pub(crate) const fn tick(&mut self) {
        self.reader.tick();
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
