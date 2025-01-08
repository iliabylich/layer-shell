mod cpu_core_info;
use crate::{global, Event};
use cpu_core_info::CpuCoreInfo;

global!(STATE, Option<Vec<CpuCoreInfo>>);

pub(crate) fn setup() {
    STATE::set(None);
}

pub(crate) fn tick() {
    let state = STATE::get();

    match CpuCoreInfo::parse_current_comparing_to(state) {
        Ok(usage_per_core) => {
            let event = Event::CpuUsage {
                usage_per_core: usage_per_core.into(),
            };
            event.emit();
        }
        Err(err) => log::error!("failed to retrieve CPU usage: {:?}", err),
    }
}
