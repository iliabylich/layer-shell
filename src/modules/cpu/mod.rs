mod cpu_core_info;

use crate::{scheduler::Actor, Event};
use anyhow::Result;
use cpu_core_info::CpuCoreInfo;
use std::{ops::ControlFlow, time::Duration};

#[derive(Debug)]
pub(crate) struct CPU {
    state: Option<Vec<CpuCoreInfo>>,
}

impl Actor for CPU {
    fn name() -> &'static str {
        "CPU"
    }

    fn start() -> Result<Box<dyn Actor>> {
        Ok(Box::new(CPU { state: None }))
    }

    fn tick(&mut self) -> Result<ControlFlow<(), Duration>> {
        let (usage, new_state) = CpuCoreInfo::parse_current_comparing_to(self.state.as_ref())?;
        let event = Event::CpuUsage {
            usage_per_core: usage.into(),
        };
        event.emit();

        self.state = Some(new_state);
        Ok(ControlFlow::Continue(Duration::from_secs(1)))
    }

    fn exec(&mut self, _: &crate::Command) -> Result<ControlFlow<()>> {
        Ok(ControlFlow::Break(()))
    }
}
