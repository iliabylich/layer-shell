pub(crate) use self::{cqe::Cqe, sqe::Sqe};
use crate::{
    sansio::{Satisfy, Wants},
    user_data::{ModuleId, UserData},
};
use generated::{
    __kernel_timespec, __liburing_cqe_seen, __liburing_get_sqe, __liburing_queue_exit,
    __liburing_queue_init, __liburing_submit, __liburing_submit_and_wait, __liburing_wait_cqe,
    __liburing_wait_cqe_timeout, io_uring, io_uring_cqe,
};
use libc::{ETIME, strerror};
use std::mem::MaybeUninit;

mod cqe;
#[expect(
    dead_code,
    unsafe_op_in_unsafe_fn,
    non_camel_case_types,
    trivial_casts,
    clippy::indexing_slicing,
    clippy::ptr_as_ptr,
    clippy::ref_as_ptr,
    clippy::missing_const_for_fn,
    clippy::use_self
)]
mod generated;
mod sqe;

fn checkerr(errno: i32) {
    if errno < 0 {
        let str = unsafe { strerror(errno) };
        let str = unsafe { std::ffi::CStr::from_ptr(str) }.to_string_lossy();
        log::error!("IoUring error: {str:?}");
        std::process::exit(1);
    }
}

static mut NOTIMEOUT: __kernel_timespec = __kernel_timespec {
    tv_sec: 0,
    tv_nsec: 0,
};

pub(crate) struct IoUring {
    ring: io_uring,
    dirty: bool,
}

impl IoUring {
    pub(crate) fn new(entries: u32, flags: u32) -> Self {
        let mut ring: io_uring = unsafe { MaybeUninit::zeroed().assume_init() };
        let errno = unsafe { __liburing_queue_init(entries, &raw mut ring, flags) };
        checkerr(errno);
        Self { ring, dirty: false }
    }

    fn get_sqe(&mut self) -> Sqe {
        let sqe = unsafe { __liburing_get_sqe(&raw mut self.ring) };
        if sqe.is_null() {
            log::error!("got NULL from io_uring_get_sqe");
            std::process::exit(1);
        }
        self.dirty = true;
        Sqe { sqe }
    }

    fn submit(&mut self) {
        let errno = unsafe { __liburing_submit(&raw mut self.ring) };
        checkerr(errno);
    }

    pub(crate) fn submit_if_dirty(&mut self) {
        if self.dirty {
            self.submit();
        }
    }

    pub(crate) fn submit_and_wait(&mut self, n: usize) {
        let errno = unsafe { __liburing_submit_and_wait(&raw mut self.ring, n as u32) };
        checkerr(errno);
    }

    #[allow(dead_code)]
    pub(crate) fn wait_cqe(&mut self) -> Cqe {
        let mut cqe: *mut io_uring_cqe = std::ptr::null_mut();
        let errno = unsafe { __liburing_wait_cqe(&raw mut self.ring, &raw mut cqe) };
        checkerr(errno);
        if cqe.is_null() {
            log::error!("got NULL from io_uring_wait_cqe");
            std::process::exit(1);
        }
        Cqe { cqe }
    }

    pub(crate) fn try_get_cqe(&mut self) -> Option<Cqe> {
        let mut cqe: *mut io_uring_cqe = std::ptr::null_mut();
        let errno = unsafe {
            __liburing_wait_cqe_timeout(&raw mut self.ring, &raw mut cqe, &raw mut NOTIMEOUT)
        };
        if errno == -ETIME {
            return None;
        }
        checkerr(errno);
        if cqe.is_null() {
            log::error!("got NULL from io_uring_wait_cqe_timeout");
            std::process::exit(1);
        }
        Some(Cqe { cqe })
    }

    pub(crate) fn cqe_seen(&mut self, cqe: Cqe) {
        unsafe { __liburing_cqe_seen(&raw mut self.ring, cqe.cqe) }
    }

    pub(crate) const fn as_raw_fd(&self) -> i32 {
        self.ring.ring_fd
    }

    pub(crate) fn deinit(&mut self) {
        unsafe { __liburing_queue_exit(&raw mut self.ring) };
    }

    pub(crate) fn schedule(&mut self, module_id: ModuleId, wants: Wants) {
        match wants {
            Wants::Socket { domain, r#type } => {
                let mut sqe = self.get_sqe();
                sqe.prep_socket(domain, r#type, 0, 0);
                sqe.set_user_data(UserData::new(module_id, Satisfy::Socket));
            }
            Wants::Connect { fd, addr, addrlen } => {
                let mut sqe = self.get_sqe();
                sqe.prep_connect(fd, addr, addrlen);
                sqe.set_user_data(UserData::new(module_id, Satisfy::Connect));
            }
            Wants::Read { fd, buf, len } => {
                let mut sqe = self.get_sqe();
                sqe.prep_read(fd, buf, len);
                sqe.set_user_data(UserData::new(module_id, Satisfy::Read));
            }
            Wants::Write { fd, buf, len } => {
                let mut sqe = self.get_sqe();
                sqe.prep_write(fd, buf, len);
                sqe.set_user_data(UserData::new(module_id, Satisfy::Write));
            }
            Wants::ReadWrite {
                fd,
                readbuf,
                readlen,
                writebuf,
                writelen,
            } => {
                let mut sqe = self.get_sqe();
                sqe.prep_read(fd, readbuf, readlen);
                sqe.set_user_data(UserData::new(module_id, Satisfy::Read));

                let mut sqe = self.get_sqe();
                sqe.prep_write(fd, writebuf, writelen);
                sqe.set_user_data(UserData::new(module_id, Satisfy::Write));
            }
            Wants::OpenAt {
                dfd,
                path,
                flags,
                mode,
            } => {
                let mut sqe = self.get_sqe();
                sqe.prep_openat(dfd, path, flags, mode);
                sqe.set_user_data(UserData::new(module_id, Satisfy::OpenAt));
            }
            Wants::Close { fd } => {
                let mut sqe = self.get_sqe();
                sqe.prep_close(fd);
                sqe.set_user_data(UserData::new(module_id, Satisfy::Close));
            }
        }
    }
}
