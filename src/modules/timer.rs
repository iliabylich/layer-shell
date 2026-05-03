use crate::{
    modules::FallibleModule,
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

impl FallibleModule for Timer {
    const NAME: &str = "Timer";
    type Output = u64;

    fn try_wants(&mut self) -> Result<Option<Wants>> {
        Ok(self.timerfd.wants())
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<Self::Output>> {
        self.timerfd.satisfy(satisfy, res).map(Some)
    }
}
