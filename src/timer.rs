use anyhow::Result;
use libc::{close, itimerspec, timerfd_create, timerfd_settime, timespec, CLOCK_MONOTONIC};

pub(crate) struct ConnectedTimer {
    fd: i32,
    ticks_count: u64,
}

impl ConnectedTimer {
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

    fn read(&mut self) {
        let mut time = 0_u64;
        let len = unsafe {
            libc::read(
                self.fd,
                (&mut time as *mut u64).cast(),
                std::mem::size_of::<u64>(),
            )
        };
        assert_eq!(len, std::mem::size_of::<u64>() as isize);
        self.ticks_count += 1;
    }

    pub(crate) fn is_multiple_of(&self, n: u64) -> bool {
        self.ticks_count.wrapping_sub(1) % n == 0
    }
}

impl Drop for ConnectedTimer {
    fn drop(&mut self) {
        unsafe {
            close(self.fd);
        }
    }
}

pub(crate) enum Timer {
    Connected(ConnectedTimer),
    Disconnected,
}

impl Timer {
    pub(crate) fn new(interval_in_sec: i64) -> Self {
        ConnectedTimer::try_new(interval_in_sec)
            .map(Self::Connected)
            .inspect_err(|err| log::error!("{:?}", err))
            .unwrap_or(Self::Disconnected)
    }

    pub(crate) fn read(&mut self) {
        if let Self::Connected(timer) = self {
            timer.read();
        }
    }

    pub(crate) fn is_multiple_of(&self, n: u64) -> bool {
        if let Self::Connected(timer) = self {
            timer.is_multiple_of(n)
        } else {
            false
        }
    }

    pub(crate) fn fd(&self) -> Option<i32> {
        if let Self::Connected(timer) = self {
            Some(timer.fd)
        } else {
            None
        }
    }
}
