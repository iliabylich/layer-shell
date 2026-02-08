use crate::macros::{assert_or_exit, report_and_exit};

pub(crate) use self::{cqe::Cqe, sqe::Sqe};
use generated::{
    __kernel_timespec, __liburing_cqe_seen, __liburing_get_sqe, __liburing_queue_exit,
    __liburing_queue_init, __liburing_submit, __liburing_submit_and_wait, __liburing_wait_cqe,
    __liburing_wait_cqe_timeout, io_uring, io_uring_cqe,
};
use libc::{ETIME, strerror};
use std::mem::MaybeUninit;

mod cqe;
#[expect(dead_code, unsafe_op_in_unsafe_fn, non_camel_case_types)]
mod generated;
mod sqe;

fn checkerr(errno: i32) {
    if errno < 0 {
        let str = unsafe { strerror(errno) };
        let str = unsafe { std::ffi::CStr::from_ptr(str) }.to_string_lossy();
        report_and_exit!("IoUring error: {str:?}")
    }
}

static mut NOTIMEOUT: __kernel_timespec = __kernel_timespec {
    tv_sec: 0,
    tv_nsec: 0,
};

pub(crate) enum IoUring {}

static mut IO_URING: io_uring = unsafe { std::mem::zeroed() };
static mut DIRTY: bool = false;

fn ring_init(entries: usize, flags: u32) {
    let mut ring: io_uring = unsafe { MaybeUninit::zeroed().assume_init() };
    let errno = unsafe { __liburing_queue_init(entries as u32, &mut ring, flags) };
    checkerr(errno);

    unsafe {
        IO_URING = ring;
        DIRTY = false;
    }
}

fn ring_get() -> &'static mut io_uring {
    #[expect(static_mut_refs)]
    unsafe {
        &mut IO_URING
    }
}

fn dirty_get() -> bool {
    unsafe { DIRTY }
}
fn dirty_set(value: bool) {
    unsafe { DIRTY = value }
}

impl IoUring {
    pub(crate) fn init(entries: usize, flags: u32) {
        ring_init(entries, flags)
    }

    pub(crate) fn get_sqe() -> Sqe {
        let sqe = unsafe { __liburing_get_sqe(ring_get()) };
        assert_or_exit!(!sqe.is_null(), "got NULL from io_uring_get_sqe");
        dirty_set(true);
        Sqe { sqe }
    }

    fn submit() {
        let errno = unsafe { __liburing_submit(ring_get()) };
        checkerr(errno)
    }

    pub(crate) fn submit_if_dirty() {
        if dirty_get() {
            Self::submit()
        }
    }

    pub(crate) fn submit_and_wait(n: usize) {
        let errno = unsafe { __liburing_submit_and_wait(ring_get(), n as u32) };
        checkerr(errno)
    }

    #[allow(dead_code)]
    pub(crate) fn wait_cqe() -> Cqe {
        let mut cqe: *mut io_uring_cqe = std::ptr::null_mut();
        let errno = unsafe { __liburing_wait_cqe(ring_get(), &mut cqe) };
        checkerr(errno);
        assert_or_exit!(!cqe.is_null(), "got NULL from io_uring_wait_cqe");
        Cqe { cqe }
    }

    pub(crate) fn try_get_cqe() -> Option<Cqe> {
        let mut cqe: *mut io_uring_cqe = std::ptr::null_mut();
        let errno =
            unsafe { __liburing_wait_cqe_timeout(ring_get(), &mut cqe, &raw mut NOTIMEOUT) };
        if errno == -ETIME {
            return None;
        }
        checkerr(errno);
        assert_or_exit!(!cqe.is_null(), "got NULL from io_uring_wait_cqe_timeout");
        Some(Cqe { cqe })
    }

    pub(crate) fn cqe_seen(cqe: Cqe) {
        unsafe { __liburing_cqe_seen(ring_get(), cqe.cqe) }
    }

    pub(crate) fn as_raw_fd() -> i32 {
        ring_get().ring_fd
    }

    pub(crate) fn deinit() {
        unsafe { __liburing_queue_exit(ring_get()) };
    }
}
