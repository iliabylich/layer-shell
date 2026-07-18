pub(crate) use self::{cqe::Cqe, sqe::Sqe};
use crate::external::{
    __kernel_timespec, __liburing_cqe_seen, __liburing_get_sqe, __liburing_queue_exit,
    __liburing_queue_init, __liburing_submit, __liburing_submit_and_wait, __liburing_wait_cqe,
    __liburing_wait_cqe_timeout, io_uring, io_uring_cqe,
};
use crate::{
    sansio::{Op, Wants},
    user_data::{ModuleId, UserData},
};
use anyhow::{Result, bail};
use core::mem::MaybeUninit;
use libc::{ETIME, exit, strerror};

mod cqe;
mod sqe;

fn checkerr(errno: i32) {
    if errno < 0 {
        let str = unsafe { strerror(errno) };
        let str = unsafe { core::ffi::CStr::from_ptr(str) }.to_string_lossy();
        log::error!("IoUring error: {str:?}");
        unsafe { exit(1) }
    }
}

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
            unsafe { exit(1) }
        }
        self.dirty = true;
        Sqe { sqe }
    }

    fn submit(&mut self) {
        let errno = unsafe { __liburing_submit(&raw mut self.ring) };
        checkerr(errno);
        self.dirty = false;
    }

    pub(crate) fn submit_if_dirty(&mut self) {
        if self.dirty {
            self.submit();
        }
    }

    pub(crate) fn submit_and_wait(&mut self, n: usize) {
        let errno = unsafe { __liburing_submit_and_wait(&raw mut self.ring, n as u32) };
        checkerr(errno);
        self.dirty = false;
    }

    #[expect(dead_code)]
    pub(crate) fn wait_cqe(&mut self) -> Result<Cqe> {
        let mut cqe: *mut io_uring_cqe = core::ptr::null_mut();
        let errno = unsafe { __liburing_wait_cqe(&raw mut self.ring, &raw mut cqe) };
        checkerr(errno);
        if cqe.is_null() {
            bail!("got NULL from io_uring_wait_cqe");
        }
        Ok(Cqe { cqe })
    }

    pub(crate) fn try_get_cqe(&mut self) -> Option<Cqe> {
        let mut cqe: *mut io_uring_cqe = core::ptr::null_mut();
        let mut notimeout = __kernel_timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        let errno = unsafe {
            __liburing_wait_cqe_timeout(&raw mut self.ring, &raw mut cqe, &raw mut notimeout)
        };
        if errno == -ETIME {
            return None;
        }
        checkerr(errno);
        if cqe.is_null() {
            log::error!("got NULL from io_uring_wait_cqe_timeout");
            unsafe { exit(1) };
        }
        Some(Cqe { cqe })
    }

    pub(crate) fn cqe_seen(&mut self, cqe: Cqe) {
        unsafe { __liburing_cqe_seen(&raw mut self.ring, cqe.cqe) }
    }

    pub(crate) const fn fd(&self) -> i32 {
        self.ring.ring_fd
    }

    pub(crate) fn deinit(&mut self) {
        unsafe { __liburing_queue_exit(&raw mut self.ring) };
    }

    pub(crate) fn schedule(&mut self, module_id: ModuleId, wants: Wants) {
        match wants {
            Wants::Socket { domain, type_, .. } => {
                let mut sqe = self.get_sqe();
                sqe.prep_socket(domain, type_, 0, 0);
                sqe.set_user_data(UserData::new(module_id, Op::Socket));
            }
            Wants::Connect { fd, addr, addrlen } => {
                let mut sqe = self.get_sqe();
                sqe.prep_connect(fd, addr, addrlen);
                sqe.set_user_data(UserData::new(module_id, Op::Connect));
            }
            Wants::Read { fd, buf, len, .. } => {
                let mut sqe = self.get_sqe();
                sqe.prep_read(fd, buf, len);
                sqe.set_user_data(UserData::new(module_id, Op::Read));
            }
            Wants::Write { fd, buf, len, .. } => {
                let mut sqe = self.get_sqe();
                sqe.prep_write(fd, buf, len);
                sqe.set_user_data(UserData::new(module_id, Op::Write));
            }
            Wants::ReadWrite {
                fd,
                readbuf,
                readlen,
                writebuf,
                writelen,
                ..
            } => {
                let mut sqe = self.get_sqe();
                sqe.prep_read(fd, readbuf, readlen);
                sqe.set_user_data(UserData::new(module_id, Op::Read));

                let mut sqe = self.get_sqe();
                sqe.prep_write(fd, writebuf, writelen);
                sqe.set_user_data(UserData::new(module_id, Op::Write));
            }
            Wants::OpenAt {
                dfd,
                path,
                flags,
                mode,
                ..
            } => {
                let mut sqe = self.get_sqe();
                sqe.prep_openat(dfd, path.as_ptr(), flags, mode);
                sqe.set_user_data(UserData::new(module_id, Op::OpenAt));
            }
            Wants::Close { fd, .. } => {
                let mut sqe = self.get_sqe();
                sqe.prep_close(fd);
                sqe.set_user_data(UserData::new(module_id, Op::Close));
            }
        }
    }
}
