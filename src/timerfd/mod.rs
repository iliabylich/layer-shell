use crate::{
    liburing::IoUring,
    macros::{assert_or_exit, define_op},
    user_data::{ModuleId, UserData},
};
use libc::{CLOCK_MONOTONIC, close, itimerspec, timerfd_create, timerfd_settime, timespec};
use std::ptr::null_mut;
pub(crate) use tick::Tick;

mod tick;

pub(crate) struct Timerfd {
    fd: i32,
    buf: [u8; 8],
    ticks: u64,
}

define_op!("TimerFD", Read);

impl Timerfd {
    pub(crate) fn new() -> Box<Self> {
        Box::new(Self {
            fd: -1,
            buf: [0; 8],
            ticks: 0,
        })
    }

    fn set_timer(&mut self) {
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

        let res = unsafe { timerfd_settime(fd, 0, &timer_spec, null_mut()) };
        assert_or_exit!(
            res != -1,
            "timerfd_settime returned -1: {}",
            std::io::Error::last_os_error()
        );

        self.fd = fd;
    }

    pub(crate) fn init(&mut self) {
        self.set_timer();
        self.schedule_read()
    }

    fn schedule_read(&mut self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_read(self.fd, self.buf.as_mut_ptr(), self.buf.len());
        sqe.set_user_data(UserData::new(ModuleId::TimerFD, Op::Read));
    }

    pub(crate) fn process(&mut self, op: u8) -> Tick {
        match Op::from(op) {
            Op::Read => {
                let ticks = self.ticks;
                self.ticks += 1;

                self.schedule_read();

                Tick(ticks)
            }
        }
    }
}

impl Drop for Timerfd {
    fn drop(&mut self) {
        unsafe { close(self.fd) };
    }
}
