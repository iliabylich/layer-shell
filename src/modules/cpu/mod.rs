mod cpu_core_info;

use crate::{
    scheduler::{Module, RepeatingModule},
    Event,
};
use anyhow::Result;
use cpu_core_info::CpuCoreInfo;
use std::time::Duration;

pub(crate) struct CPU {
    state: Option<Vec<CpuCoreInfo>>,
}

impl Module for CPU {
    const NAME: &str = "CPU";

    fn start() -> Result<Option<Box<dyn RepeatingModule>>> {
        Ok(Some(Box::new(CPU { state: None })))
    }
}

impl RepeatingModule for CPU {
    fn tick(&mut self) -> Result<Duration> {
        let (usage, new_state) = CpuCoreInfo::parse_current_comparing_to(self.state.as_ref())?;
        let event = Event::CpuUsage {
            usage_per_core: usage.into(),
        };
        event.emit();

        self.state = Some(new_state);

        Ok(Duration::from_secs(1))
    }

    fn exec(&mut self, _: &crate::Command) -> Result<()> {
        Ok(())
    }
}
