use crate::sansio::{Satisfy, TimerFd, Wants};

#[derive(Debug, Clone, Copy)]
pub struct Timer {
    timerfd: TimerFd,
}

impl Timer {
    pub(crate) fn new() -> Option<Self> {
        log::trace!("Creating Timer");

        match TimerFd::new() {
            Ok(timerfd) => Some(Self { timerfd }),
            Err(()) => None,
        }
    }

    pub(crate) fn wants(&mut self, buf: &mut [u8; 8]) -> Option<Wants> {
        let wants = self.timerfd.wants(buf)?;
        log::trace!("{wants:?}");
        Some(wants)
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, buf: [u8; 8]) -> Result<Option<u64>, ()> {
        self.timerfd.try_satisfy(satisfy, buf)
    }
}
