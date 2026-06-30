pub(crate) use self::{cqe::Cqe, sqe::Sqe};
use crate::{
    sansio::{Op, Wants},
    user_data::{ModuleId, UserData},
};
use alloc::ffi::CString;
use core::mem::MaybeUninit;
use generated::{
    __kernel_timespec, __liburing_cqe_seen, __liburing_get_sqe, __liburing_queue_exit,
    __liburing_queue_init, __liburing_submit, __liburing_submit_and_wait, __liburing_wait_cqe,
    __liburing_wait_cqe_timeout, io_uring, io_uring_cqe,
};
use libc::{ETIME, strerror};
use rustix::net::SocketAddrAny;
use std::{
    collections::{HashMap, HashSet},
    os::fd::AsRawFd,
};

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
    clippy::use_self,
    clippy::std_instead_of_core
)]
mod generated;
mod sqe;

fn checkerr(errno: i32) {
    if errno < 0 {
        let str = unsafe { strerror(errno) };
        let str = unsafe { core::ffi::CStr::from_ptr(str) }.to_string_lossy();
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
    socket_addr_cache: HashSet<Box<SocketAddrAny>>,
    string_cache: HashMap<&'static str, CString>,
}

impl IoUring {
    pub(crate) fn new(entries: u32, flags: u32) -> Self {
        let mut ring: io_uring = unsafe { MaybeUninit::zeroed().assume_init() };
        let errno = unsafe { __liburing_queue_init(entries, &raw mut ring, flags) };
        checkerr(errno);
        Self {
            ring,
            dirty: false,
            socket_addr_cache: HashSet::new(),
            string_cache: HashMap::new(),
        }
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
    pub(crate) fn wait_cqe(&mut self) -> Cqe {
        let mut cqe: *mut io_uring_cqe = core::ptr::null_mut();
        let errno = unsafe { __liburing_wait_cqe(&raw mut self.ring, &raw mut cqe) };
        checkerr(errno);
        if cqe.is_null() {
            log::error!("got NULL from io_uring_wait_cqe");
            std::process::exit(1);
        }
        Cqe { cqe }
    }

    pub(crate) fn try_get_cqe(&mut self) -> Option<Cqe> {
        let mut cqe: *mut io_uring_cqe = core::ptr::null_mut();
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
            Wants::Socket { domain, r#type, .. } => {
                let mut sqe = self.get_sqe();
                sqe.prep_socket(
                    i32::from(domain.as_raw()),
                    i32::try_from(r#type.as_raw()).unwrap_or_else(|_| unreachable!()),
                    0,
                    0,
                );
                sqe.set_user_data(UserData::new(module_id, Op::Socket));
            }
            Wants::Connect { fd, addr, .. } => {
                let mut sqe = self.get_sqe();
                if !self.socket_addr_cache.contains(&addr) {
                    self.socket_addr_cache.insert(Box::new(addr.clone()));
                }
                let addr = self
                    .socket_addr_cache
                    .get(&addr)
                    .unwrap_or_else(|| unreachable!());
                sqe.prep_connect(fd.as_raw_fd(), addr.as_ptr().cast(), addr.addr_len());
                sqe.set_user_data(UserData::new(module_id, Op::Connect));
            }
            Wants::Read { fd, buf, len, .. } => {
                let mut sqe = self.get_sqe();
                sqe.prep_read(fd.as_raw_fd(), buf, len);
                sqe.set_user_data(UserData::new(module_id, Op::Read));
            }
            Wants::Write { fd, buf, len, .. } => {
                let mut sqe = self.get_sqe();
                sqe.prep_write(fd.as_raw_fd(), buf, len);
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
                sqe.prep_read(fd.as_raw_fd(), readbuf, readlen);
                sqe.set_user_data(UserData::new(module_id, Op::Read));

                let mut sqe = self.get_sqe();
                sqe.prep_write(fd.as_raw_fd(), writebuf, writelen);
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
                if !self.string_cache.contains_key(path) {
                    self.string_cache
                        .insert(path, CString::new(path).unwrap_or_else(|_| unreachable!()));
                }
                let path = self
                    .string_cache
                    .get(path)
                    .unwrap_or_else(|| unreachable!());

                sqe.prep_openat(
                    dfd.as_raw_fd(),
                    path.as_ptr(),
                    i32::try_from(flags.bits()).unwrap_or_else(|_| unreachable!()),
                    mode.bits(),
                );
                sqe.set_user_data(UserData::new(module_id, Op::OpenAt));
            }
            Wants::Close { fd, .. } => {
                let mut sqe = self.get_sqe();
                sqe.prep_close(fd.as_raw_fd());
                sqe.set_user_data(UserData::new(module_id, Op::Close));
            }
        }
    }
}
