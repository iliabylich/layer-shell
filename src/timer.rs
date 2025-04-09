use crate::{channel::EventSender, fd_id::FdId, modules::Module};
use anyhow::Result;
use rustix::{
    io::read,
    time::{
        Itimerspec, TimerfdClockId, TimerfdFlags, TimerfdTimerFlags, Timespec, timerfd_create,
        timerfd_settime,
    },
};
use std::os::fd::{AsRawFd, OwnedFd, RawFd};

pub(crate) struct Timer {
    fd: OwnedFd,
    ticks_count: u64,
}

impl Module for Timer {
    const FD_ID: FdId = FdId::Timer;
    const NAME: &str = "Timer";

    type ReadOutput = Ticks;

    fn new(_: EventSender) -> Result<Self> {
        let fd = timerfd_create(TimerfdClockId::Realtime, TimerfdFlags::empty())?;

        let timerspec = Itimerspec {
            it_interval: Timespec {
                tv_sec: 1,
                tv_nsec: 0,
            },
            it_value: Timespec {
                tv_sec: 0,
                tv_nsec: 1,
            },
        };
        timerfd_settime(&fd, TimerfdTimerFlags::empty(), &timerspec)?;

        Ok(Self { fd, ticks_count: 0 })
    }

    fn read_events(&mut self) -> Result<Ticks> {
        let mut buf = [0; 50];
        let len = read(&self.fd, &mut buf)?;
        anyhow::ensure!(len == std::mem::size_of::<u64>());
        self.ticks_count += 1;
        Ok(Ticks {
            ticks_count: self.ticks_count,
        })
    }
}

impl AsRawFd for Timer {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Ticks {
    ticks_count: u64,
}

impl Ticks {
    pub(crate) fn is_multiple_of(&self, n: u64) -> bool {
        self.ticks_count.wrapping_sub(1) % n == 0
    }
}
