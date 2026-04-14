use crate::{
    sansio::{Satisfy, TimerFd, Wants},
    user_data::ModuleId,
    utils::report_and_exit,
};

pub(crate) struct Timer {
    timerfd: TimerFd,
}

impl Timer {
    pub(crate) fn new() -> Self {
        Self {
            timerfd: TimerFd::new(),
        }
    }

    pub(crate) const fn module_id(&self) -> ModuleId {
        ModuleId::Timer
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        self.timerfd.wants()
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> u64 {
        self.timerfd
            .satisfy(satisfy, res)
            .unwrap_or_else(|err| report_and_exit!("{err:?}"))
    }
}
