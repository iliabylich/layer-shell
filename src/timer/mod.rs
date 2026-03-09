use crate::{
    macros::report_and_exit,
    modules::Module,
    sansio::{Satisfy, TimerFd, Wants},
    user_data::ModuleId,
};
use anyhow::Result;
use std::convert::Infallible;

pub(crate) struct Timer {
    timerfd: TimerFd,
}

impl Module for Timer {
    type Input = ();
    type Output = u64;
    type Error = Infallible;

    const MODULE_ID: ModuleId = ModuleId::Timer;

    fn new((): Self::Input) -> Self {
        Self {
            timerfd: TimerFd::new(),
        }
    }

    fn wants(&mut self) -> Wants {
        self.timerfd.wants()
    }

    fn satisfy(
        &mut self,
        satisfy: Satisfy,
        res: i32,
        _events: &mut Vec<crate::Event>,
    ) -> Result<Self::Output, Self::Error> {
        let tick = self
            .timerfd
            .satisfy(satisfy, res)
            .unwrap_or_else(|err| report_and_exit!("{err:?}"));
        Ok(tick)
    }

    fn tick(&mut self, _tick: u64) {
        unreachable!("timer procudes ticks, but doesn't consume")
    }
}
