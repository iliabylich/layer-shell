use anyhow::{bail, Result};
use libc::{close, itimerspec, timerfd_create, timerfd_settime, timespec, CLOCK_MONOTONIC};
use std::os::fd::AsRawFd;

pub(crate) struct Timer {
    fd: i32,
    ticks_count: u64,
}

impl Timer {
    pub(crate) fn new(interval_in_sec: i64) -> Result<Self> {
        let fd = unsafe { timerfd_create(CLOCK_MONOTONIC, 0) };
        if fd == -1 {
            return Err(anyhow::Error::from(std::io::Error::last_os_error())
                .context("timerfd_create failed"));
        }
        let timer = Self { fd, ticks_count: 0 };

        let timerspec = itimerspec {
            it_interval: timespec {
                tv_sec: interval_in_sec,
                tv_nsec: 0,
            },
            it_value: timespec {
                tv_sec: 0,
                tv_nsec: 1,
            },
        };
        let res = unsafe { timerfd_settime(fd, 0, &timerspec, std::ptr::null_mut()) };
        if res == -1 {
            return Err(anyhow::Error::from(std::io::Error::last_os_error())
                .context("timerfd_settime failed"));
        }

        Ok(timer)
    }

    pub(crate) fn read(&mut self) -> Result<()> {
        let mut time = 0_u64;
        let len = unsafe {
            libc::read(
                self.fd,
                (&mut time as *mut u64).cast(),
                std::mem::size_of::<u64>(),
            )
        };
        if len != std::mem::size_of::<u64>() as isize {
            bail!("failed to read 8 bytes representing timer tick");
        }
        self.ticks_count += 1;
        Ok(())
    }

    pub(crate) fn ticks_count(&self) -> u64 {
        self.ticks_count.wrapping_sub(1)
    }

    pub(crate) fn is_multiple_of(&self, n: u64) -> bool {
        self.ticks_count() % n == 0
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        unsafe { close(self.fd) };
    }
}

impl AsRawFd for Timer {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        self.fd
    }
}
