mod cpu_core_info;

use crate::{scheduler::Module, Event};
use anyhow::{Context as _, Result};
use cpu_core_info::CpuCoreInfo;
use std::any::Any;

type State = Vec<CpuCoreInfo>;

pub(crate) struct CPU;

impl Module for CPU {
    const NAME: &str = "CPU";
    const INTERVAL: Option<u64> = Some(1_000);

    fn start() -> Result<Box<dyn Any + Send + 'static>> {
        Ok(Box::new(Option::<State>::None))
    }

    fn tick(state: &mut Box<dyn Any + Send + 'static>) -> Result<()> {
        let state = state
            .downcast_mut::<Option<State>>()
            .context("CPU state is malformed")?;

        let (usage, new_state) = CpuCoreInfo::parse_current_comparing_to(state.as_ref())?;
        let event = Event::CpuUsage {
            usage_per_core: usage.into(),
        };
        event.emit();

        *state = Some(new_state);

        Ok(())
    }
}
