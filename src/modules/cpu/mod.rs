mod cpu_core_info;

use crate::{scheduler::Actor, Event};
use anyhow::{Context as _, Result};
use cpu_core_info::CpuCoreInfo;
use std::{ops::ControlFlow, sync::mpsc::Sender, time::Duration};

#[derive(Debug)]
pub(crate) struct CPU {
    state: Option<Vec<CpuCoreInfo>>,
    tx: Sender<Event>,
}

impl Actor for CPU {
    fn name() -> &'static str {
        "CPU"
    }

    fn start(tx: Sender<Event>) -> Result<Box<dyn Actor>> {
        Ok(Box::new(CPU { state: None, tx }))
    }

    fn tick(&mut self) -> Result<ControlFlow<(), Duration>> {
        let (usage, new_state) = CpuCoreInfo::parse_current_comparing_to(self.state.as_ref())?;
        let event = Event::CpuUsage {
            usage_per_core: usage.into(),
        };
        self.tx
            .send(event)
            .context("failed to send CpuUsage event")?;

        self.state = Some(new_state);
        Ok(ControlFlow::Continue(Duration::from_secs(1)))
    }

    fn exec(&mut self, _: &crate::Command) -> Result<ControlFlow<()>> {
        Ok(ControlFlow::Break(()))
    }
}
