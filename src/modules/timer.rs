use crate::{
    modules::Module,
    sansio::{Satisfy, TimerFd, Wants},
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
}

impl Module for Timer {
    type Output = Result<u64>;

    fn wants(&mut self) -> Result<Option<Wants>> {
        Ok(self.timerfd.wants())
    }

    fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Self::Output {
        self.timerfd.satisfy(satisfy, res)
    }
}
