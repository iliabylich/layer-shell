use crate::{
    sansio::{Satisfy, Wants},
    utils::assert_or_exit,
};
use anyhow::{Result, bail, ensure};
use libc::{CLOCK_MONOTONIC, itimerspec, timerfd_create, timerfd_settime, timespec};

pub(crate) struct TimerFd {
    fd: i32,
    buf: [u8; 8],
    ticks: u64,
    state: State,
}

enum State {
    CanRead,
    WaitingForRead,
}

impl TimerFd {
    pub(crate) fn new() -> Self {
        Self {
            fd: create_timer(),
            buf: [0; _],
            ticks: 0,
            state: State::CanRead,
        }
    }

    pub(crate) fn wants(&mut self) -> Wants {
        match self.state {
            State::CanRead => {
                self.state = State::WaitingForRead;
                Wants::Read {
                    fd: self.fd,
                    buf: self.buf.as_mut_ptr(),
                    len: self.buf.len(),
                }
            }
            State::WaitingForRead => Wants::Nothing,
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<u64> {
        match satisfy {
            Satisfy::Read => {
                ensure!(res as usize == self.buf.len());
                let expirations = u64::from_ne_bytes(self.buf);

                let ticks = self.ticks;
                self.ticks = self.ticks.saturating_add(expirations);
                self.state = State::CanRead;

                Ok(ticks)
            }

            unsupported => bail!("unexpected satisfy for TimerFd: {unsupported:?}"),
        }
    }
}

fn create_timer() -> i32 {
    let fd = unsafe { timerfd_create(CLOCK_MONOTONIC, 0) };

    assert_or_exit!(
        fd != -1,
        "timerfd_create returned -1: {}",
        std::io::Error::last_os_error()
    );

    let timer_spec = itimerspec {
        it_interval: timespec {
            tv_sec: 1,
            tv_nsec: 0,
        },
        it_value: timespec {
            tv_sec: 0,
            tv_nsec: 1,
        },
    };

    let res = unsafe { timerfd_settime(fd, 0, &timer_spec, core::ptr::null_mut()) };
    assert_or_exit!(
        res != -1,
        "timerfd_settime returned -1: {}",
        std::io::Error::last_os_error()
    );

    fd
}
