use crate::{
    error::IoError,
    sansio::{Satisfy, TimerFd, Wants},
};

#[derive(Debug, Clone, Copy)]
pub struct Timer {
    timerfd: TimerFd,
}

impl Timer {
    pub(crate) fn new() -> Option<Self> {
        match TimerFd::new() {
            Ok(timerfd) => Some(Self { timerfd }),
            Err(err) => {
                log::error!(target: "Timer", "{err:?}");
                None
            }
        }
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
