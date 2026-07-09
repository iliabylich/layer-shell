use crate::sansio::{Satisfy, TimerFd, Wants};
use anyhow::Result;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Timer {
    timerfd: TimerFd,
}

impl Timer {
    pub(crate) fn new() -> Result<Self> {
        Ok(Self {
            timerfd: TimerFd::new()?,
        })
    }

    pub(crate) const fn wants(&mut self, buf: &mut [u8; 8]) -> Option<Wants> {
        self.timerfd.wants(buf)
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, buf: [u8; 8]) -> Result<Option<u64>> {
        self.timerfd.try_satisfy(satisfy, buf)
    }
}
