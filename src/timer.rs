use crate::{
    epoll::{FdId, Reader},
    modules::maybe_connected::MaybeConnected,
};
use anyhow::Result;
use libc::{CLOCK_MONOTONIC, close, itimerspec, timerfd_create, timerfd_settime, timespec};

pub(crate) struct Timer {
    fd: i32,
    ticks_count: u64,
}

impl Timer {
    fn try_new(interval_in_sec: i64) -> Result<Self> {
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

    pub(crate) fn new(interval_in_sec: i64) -> MaybeConnected<Self> {
        MaybeConnected::new(Self::try_new(interval_in_sec))
    }
}

impl Reader for Timer {
    type Output = Ticks;

    const NAME: &str = "Timer";

    fn read(&mut self) -> Result<Self::Output> {
        let mut time = 0_u64;
        let len = unsafe {
            libc::read(
                self.fd,
                (&mut time as *mut u64).cast(),
                std::mem::size_of::<u64>(),
            )
        };
        anyhow::ensure!(len == std::mem::size_of::<u64>() as isize);
        self.ticks_count += 1;
        Ok(Ticks {
            ticks_count: self.ticks_count,
        })
    }

    fn fd(&self) -> i32 {
        self.fd
    }

    fn fd_id(&self) -> FdId {
        FdId::Timer
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        unsafe {
            close(self.fd);
        }
    }
}

pub(crate) struct Ticks {
    ticks_count: u64,
}

impl Ticks {
    pub(crate) fn is_multiple_of(&self, n: u64) -> bool {
        self.ticks_count.wrapping_sub(1) % n == 0
    }
}
