mod cpu_core_info;

use crate::{channel::VerboseSender, Event};
use anyhow::Result;
use cpu_core_info::CpuCoreInfo;

pub(crate) struct CPU {
    state: Option<Vec<CpuCoreInfo>>,
    tx: VerboseSender<Event>,
    buf: Vec<u8>,
}

impl CPU {
    pub(crate) const INTERVAL: u64 = 1;

    pub(crate) fn new(tx: VerboseSender<Event>) -> Self {
        Self {
            tx,
            state: None,
            buf: vec![0; 1_000],
        }
    }

    pub(crate) fn tick(&mut self) {
        if let Err(err) = self.try_tick() {
            log::error!("failed to tick CPU: {:?}", err);
        }
    }

    fn try_tick(&mut self) -> Result<()> {
        let (usage, new_state) =
            CpuCoreInfo::parse_current_comparing_to(self.state.as_ref(), &mut self.buf)?;
        let event = Event::CpuUsage {
            usage_per_core: usage.into(),
        };
        self.tx.send(event);
        self.state = Some(new_state);
        Ok(())
    }
}
