use crate::{
    sansio::{Satisfy, TimerFd, Wants},
    user_data::ModuleId,
};
use anyhow::Result;

pub(crate) struct Timer {
    timerfd: TimerFd,
}

impl Timer {
    pub(crate) fn new() -> Result<Self> {
        Ok(Self {
            timerfd: TimerFd::new()?,
        })
    }

    pub(crate) const fn module_id(&self) -> ModuleId {
        ModuleId::Timer
    }

    pub(crate) fn wants(&mut self) -> Result<Option<Wants>> {
        Ok(self.timerfd.wants())
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<u64> {
        self.timerfd.satisfy(satisfy, res)
    }
}
