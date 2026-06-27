use crate::{
    actor::{CanStop, TryWantsTrySatisfy},
    event_queue::EventQueue,
    sansio::{Satisfy, TimerFd, Wants},
    user_data::ModuleId,
};
use anyhow::Result;

pub(crate) enum Timer {
    Running(TimerFd),
    Stopped,
}

impl Timer {
    pub(crate) fn new() -> Self {
        Self::Running(TimerFd::new())
    }
}

impl TryWantsTrySatisfy for Timer {
    const ID: ModuleId = ModuleId::Timer;
    type Output = Option<u64>;

    fn try_wants(&mut self) -> Result<Option<Wants>> {
        match self {
            Timer::Running(timerfd) => Ok(timerfd.wants()),
            Timer::Stopped => Ok(None),
        }
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, _events: &mut EventQueue) -> Result<Self::Output> {
        match self {
            Timer::Running(timerfd) => timerfd.try_satisfy(satisfy),
            Timer::Stopped => Ok(None),
        }
    }
}

impl CanStop for Timer {
    fn stopped(&mut self) -> Self {
        Self::Stopped
    }
}
