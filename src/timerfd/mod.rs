use crate::liburing::{Cqe, IoUring};
use anyhow::{Result, ensure};
use libc::{CLOCK_MONOTONIC, close, itimerspec, timerfd_create, timerfd_settime, timespec};
use std::ptr::null_mut;
use tick::Tick;

mod tick;

#[derive(Debug)]
enum State {
    CanRead,
    Reading,
}

pub(crate) struct Timerfd {
    fd: i32,
    buf: Box<[u8; 8]>,
    read_user_data: u64,
    ticks: u64,
    state: State,
}

impl Timerfd {
    pub(crate) fn new(read_user_data: u64) -> Result<Self> {
        let fd = unsafe { timerfd_create(CLOCK_MONOTONIC, 0) };
        ensure!(
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
        let this = Self {
            fd,
            buf: Box::new([0; 8]),
            read_user_data,
            ticks: 0,
            state: State::CanRead,
        };

        let res = unsafe { timerfd_settime(this.fd, 0, &timer_spec, null_mut()) };
        ensure!(
            res != -1,
            "timerfd_settime returned -1: {}",
            std::io::Error::last_os_error()
        );

        Ok(this)
    }

    pub(crate) fn drain(&mut self, ring: &mut IoUring) -> Result<bool> {
        match self.state {
            State::CanRead => {
                let mut sqe = ring.get_sqe()?;
                sqe.prep_read(self.fd, self.buf.as_mut_ptr(), self.buf.len());
                sqe.set_user_data(self.read_user_data);

                self.state = State::Reading;
                Ok(true)
            }
            State::Reading => Ok(false),
        }
    }

    pub(crate) fn feed(&mut self, cqe: Cqe) -> Result<Option<Tick>> {
        if cqe.user_data() == self.read_user_data {
            ensure!(
                matches!(self.state, State::Reading),
                "malformed state, expected Reading, got {:?}",
                self.state
            );
            let ticks = self.ticks;
            self.ticks += 1;
            self.state = State::CanRead;
            return Ok(Some(Tick(ticks)));
        }

        Ok(None)
    }
}

impl Drop for Timerfd {
    fn drop(&mut self) {
        unsafe { close(self.fd) };
    }
}
