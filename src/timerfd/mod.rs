use crate::{
    liburing::IoUring,
    user_data::{ModuleId, UserData},
};
use anyhow::{Result, ensure};
use libc::{CLOCK_MONOTONIC, close, itimerspec, timerfd_create, timerfd_settime, timespec};
use std::ptr::null_mut;
pub(crate) use tick::Tick;

mod tick;

pub(crate) struct Timerfd {
    fd: i32,
    buf: [u8; 8],
    ticks: u64,
}

#[derive(Debug)]
#[repr(u8)]
enum Op {
    Read = 1,
}
const MAX_OP: u8 = Op::Read as u8;

impl TryFrom<u8> for Op {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self> {
        ensure!(value <= MAX_OP);
        unsafe { Ok(std::mem::transmute::<u8, Self>(value)) }
    }
}

impl Timerfd {
    pub(crate) fn new() -> Result<Box<Self>> {
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
            buf: [0; 8],
            ticks: 0,
        };

        let res = unsafe { timerfd_settime(this.fd, 0, &timer_spec, null_mut()) };
        ensure!(
            res != -1,
            "timerfd_settime returned -1: {}",
            std::io::Error::last_os_error()
        );

        Ok(Box::new(this))
    }

    pub(crate) fn init(&mut self, ring: &mut IoUring) -> Result<()> {
        self.schedule_read(ring)
    }

    fn schedule_read(&mut self, ring: &mut IoUring) -> Result<()> {
        let mut sqe = ring.get_sqe()?;
        sqe.prep_read(self.fd, self.buf.as_mut_ptr(), self.buf.len());
        sqe.set_user_data(UserData::new(ModuleId::TimerFD, Op::Read as u8));
        Ok(())
    }

    pub(crate) fn process(&mut self, op: u8, ring: &mut IoUring) -> Result<Tick> {
        match Op::try_from(op)? {
            Op::Read => {
                let ticks = self.ticks;
                self.ticks += 1;

                self.schedule_read(ring)?;

                Ok(Tick(ticks))
            }
        }
    }
}

impl Drop for Timerfd {
    fn drop(&mut self) {
        unsafe { close(self.fd) };
    }
}
