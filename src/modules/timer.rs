use crate::{
    error::IoError,
    sansio::{Satisfy, TimerFd, Wants},
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Timer {
    timerfd: TimerFd,
}

impl Timer {
    pub(crate) fn new() -> Result<Self, IoError> {
        Ok(Self {
            timerfd: TimerFd::new()?,
        })
    }

    pub(crate) const fn wants(&mut self, buf: &mut [u8; 8]) -> Option<Wants> {
        self.timerfd.wants(buf)
    }

    pub(crate) fn satisfy(
        &mut self,
        satisfy: Satisfy,
        buf: [u8; 8],
    ) -> Result<Option<u64>, IoError> {
        self.timerfd.try_satisfy(satisfy, buf)
    }
}
