use crate::sansio::{Satisfy, Wants};
use anyhow::{Context, Result, bail, ensure};
use libc::{CLOCK_MONOTONIC, itimerspec, timerfd_create, timerfd_settime, timespec};
use std::os::fd::BorrowedFd;

pub(crate) struct TimerFd {
    fd: BorrowedFd<'static>,
    buf: [u8; 8],
    ticks: u64,
    state: State,
}

#[derive(Debug, Clone, Copy)]
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

    pub(crate) const fn wants(&mut self) -> Option<Wants> {
        match self.state {
            State::CanRead => {
                self.state = State::WaitingForRead;

                Some(Wants::Read {
                    fd: self.fd,
                    buf: self.buf.as_mut_ptr(),
                    len: self.buf.len(),
                })
            }
            State::WaitingForRead => None,
        }
    }

    pub(crate) fn try_satisfy(&mut self, satisfy: Satisfy) -> Result<Option<u64>> {
        match (self.state, satisfy) {
            (State::WaitingForRead, Satisfy::Read(len)) => {
                let bytes_read = len.context("TimerFd: read failed")?;
                ensure!(bytes_read == self.buf.len());
                let expirations = u64::from_ne_bytes(self.buf);

                let ticks = self.ticks;
                self.ticks = self.ticks.saturating_add(expirations);
                self.state = State::CanRead;

                Ok(Some(ticks))
            }

            (state, unsupported) => {
                bail!("unexpected satisfy for TimerFd in {state:?}: {unsupported:?}")
            }
        }
    }
}

fn create_timer() -> BorrowedFd<'static> {
    let fd = unsafe { timerfd_create(CLOCK_MONOTONIC, 0) };

    if fd == -1 {
        log::error!(
            "timerfd_create returned -1: {}",
            std::io::Error::last_os_error()
        );
        std::process::exit(1);
    }

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

    let res = unsafe { timerfd_settime(fd, 0, &raw const timer_spec, core::ptr::null_mut()) };

    if res == -1 {
        log::error!(
            "timerfd_settime returned -1: {}",
            std::io::Error::last_os_error()
        );
        std::process::exit(1);
    }

    unsafe { BorrowedFd::borrow_raw(fd) }
}
