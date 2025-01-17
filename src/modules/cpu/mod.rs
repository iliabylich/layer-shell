mod cpu_core_info;
use crate::{fatal::fatal, Event};
use cpu_core_info::CpuCoreInfo;
use std::sync::{LazyLock, Mutex};

static STATE: LazyLock<Mutex<Option<Vec<CpuCoreInfo>>>> = LazyLock::new(|| Mutex::new(None));

pub(crate) fn tick() {
    let state = {
        STATE
            .lock()
            .unwrap_or_else(|_| fatal!("lock is poisoned"))
            .clone()
    };

    match CpuCoreInfo::parse_current_comparing_to(state) {
        Ok((usage, new_state)) => {
            let event = Event::CpuUsage {
                usage_per_core: usage.into(),
            };
            event.emit();

            let mut state = STATE.lock().unwrap_or_else(|_| fatal!("lock is poisoned"));
            *state = Some(new_state);
        }
        Err(err) => log::error!("failed to retrieve CPU usage: {:?}", err),
    }
}
