pub(crate) use self::{cqe::Cqe, sqe::Sqe};
use anyhow::{Result, bail};
use generated::{
    __io_uring_cqe_seen, __io_uring_get_sqe, __io_uring_queue_exit, __io_uring_queue_init,
    __io_uring_submit, __io_uring_submit_and_wait, __io_uring_wait_cqe,
    __io_uring_wait_cqe_timeout, __kernel_timespec, io_uring, io_uring_cqe,
};
use libc::{ETIME, strerror};
use std::{mem::MaybeUninit, os::fd::AsRawFd};

mod cqe;
#[expect(
    dead_code,
    unsafe_op_in_unsafe_fn,
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case
)]
mod generated;
mod sqe;

fn checkerr(errno: i32) -> Result<()> {
    if errno >= 0 {
        Ok(())
    } else {
        let str = unsafe { strerror(errno) };
        let str = unsafe { std::ffi::CStr::from_ptr(str) }.to_string_lossy();
        bail!("{str}")
    }
}

static mut NOTIMEOUT: __kernel_timespec = __kernel_timespec {
    tv_sec: 0,
    tv_nsec: 0,
};

pub(crate) struct IoUring {
    ring: io_uring,
}

impl IoUring {
    pub(crate) fn new(entries: usize, flags: u32) -> Result<Self> {
        let mut ring: io_uring = unsafe { MaybeUninit::zeroed().assume_init() };
        let errno = unsafe { __io_uring_queue_init(entries as u32, &mut ring, flags) };
        checkerr(errno)?;
        Ok(Self { ring })
    }

    pub(crate) fn get_sqe(&mut self) -> Result<Sqe> {
        let sqe = unsafe { __io_uring_get_sqe(&mut self.ring) };
        if sqe.is_null() {
            bail!("got NULL from io_uring_get_sqe");
        }
        Ok(Sqe { sqe })
    }

    pub(crate) fn submit(&mut self) -> Result<()> {
        let errno = unsafe { __io_uring_submit(&mut self.ring) };
        checkerr(errno)
    }

    pub(crate) fn submit_and_wait(&mut self, n: usize) -> Result<()> {
        let errno = unsafe { __io_uring_submit_and_wait(&mut self.ring, n as u32) };
        checkerr(errno)
    }

    #[allow(dead_code)]
    pub(crate) fn wait_cqe(&mut self) -> Result<Cqe> {
        let mut cqe: *mut io_uring_cqe = std::ptr::null_mut();
        let errno = unsafe { __io_uring_wait_cqe(&mut self.ring, &mut cqe) };
        checkerr(errno)?;
        if cqe.is_null() {
            bail!("got NULL from io_uring_wait_cqe");
        }
        Ok(Cqe { cqe })
    }

    pub(crate) fn try_get_cqe(&mut self) -> Result<Option<Cqe>> {
        let mut cqe: *mut io_uring_cqe = std::ptr::null_mut();
        let errno =
            unsafe { __io_uring_wait_cqe_timeout(&mut self.ring, &mut cqe, &raw mut NOTIMEOUT) };
        if errno == -ETIME {
            return Ok(None);
        }
        checkerr(errno)?;
        if cqe.is_null() {
            bail!("got NULL from io_uring_wait_cqe_timeout");
        }
        Ok(Some(Cqe { cqe }))
    }

    pub(crate) fn cqe_seen(&mut self, cqe: Cqe) {
        unsafe { __io_uring_cqe_seen(&mut self.ring, cqe.cqe) }
    }
}

impl Drop for IoUring {
    fn drop(&mut self) {
        unsafe { __io_uring_queue_exit(&mut self.ring) };
    }
}

impl AsRawFd for IoUring {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        self.ring.ring_fd
    }
}
