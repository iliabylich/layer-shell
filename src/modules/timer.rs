use crate::{
    event_queue::EventQueue,
    sansio::{Satisfy, TimerFd, Wants},
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

    pub(crate) const fn wants(&mut self) -> Option<Wants> {
        self.timerfd.wants()
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, _events: &mut EventQueue) -> Option<u64> {
        self.timerfd.satisfy(satisfy)
    }
}
