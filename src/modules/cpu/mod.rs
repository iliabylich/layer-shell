mod cpu_core_info;

use crate::{scheduler::Module, Event};
use anyhow::{anyhow, Result};
use cpu_core_info::CpuCoreInfo;
use std::sync::{LazyLock, Mutex};

static STATE: LazyLock<Mutex<Option<Vec<CpuCoreInfo>>>> = LazyLock::new(|| Mutex::new(None));

pub(crate) struct CPU;

impl Module for CPU {
    const NAME: &str = "CPU";

    fn start() -> Result<Option<(u64, fn() -> Result<()>)>> {
        Ok(Some((1_000, tick)))
    }
}

fn tick() -> Result<()> {
    let mut state = STATE.lock().map_err(|_| anyhow!("lock is poisoned"))?;

    let (usage, new_state) = CpuCoreInfo::parse_current_comparing_to(state.as_ref())?;
    let event = Event::CpuUsage {
        usage_per_core: usage.into(),
    };
    event.emit();

    *state = Some(new_state);

    Ok(())
}
