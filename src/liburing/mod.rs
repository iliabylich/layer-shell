pub(crate) use self::{cqe::Cqe, sqe::Sqe};
use crate::{
    sansio::{Op, Wants},
    user_data::{ModuleId, UserData},
};
use anyhow::{Result, bail};
use core::mem::MaybeUninit;
use generated::{
    __kernel_timespec, __liburing_cqe_seen, __liburing_get_sqe, __liburing_queue_exit,
    __liburing_queue_init, __liburing_submit, __liburing_submit_and_wait, __liburing_wait_cqe,
    __liburing_wait_cqe_timeout, io_uring, io_uring_cqe,
};
use libc::{ETIME, strerror};
use rustix::net::SocketAddrAny;

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

fn checkerr(errno: i32) -> Result<()> {
    if errno < 0 {
        let str = unsafe { strerror(errno) };
        let str = unsafe { core::ffi::CStr::from_ptr(str) }.to_string_lossy();
        bail!("IoUring error: {str:?}");
    }
    Ok(())
}

static mut NOTIMEOUT: __kernel_timespec = __kernel_timespec {
    tv_sec: 0,
    tv_nsec: 0,
};

pub(crate) struct IoUring {
    ring: io_uring,
    dirty: bool,
    #[expect(clippy::vec_box)]
    socket_addr_cache: Vec<Box<SocketAddrAny>>,
}

impl IoUring {
    pub(crate) fn new(entries: u32, flags: u32) -> Result<Self> {
        let mut ring: io_uring = unsafe { MaybeUninit::zeroed().assume_init() };
        let errno = unsafe { __liburing_queue_init(entries, &raw mut ring, flags) };
        checkerr(errno)?;
        Ok(Self {
            ring,
            dirty: false,
            socket_addr_cache: vec![],
        })
    }

    fn get_sqe(&mut self) -> Result<Sqe> {
        let sqe = unsafe { __liburing_get_sqe(&raw mut self.ring) };
        if sqe.is_null() {
            bail!("got NULL from io_uring_get_sqe");
        }
        self.dirty = true;
        Ok(Sqe { sqe })
    }

    fn submit(&mut self) -> Result<()> {
        let errno = unsafe { __liburing_submit(&raw mut self.ring) };
        checkerr(errno)?;
        self.dirty = false;
        Ok(())
    }

    pub(crate) fn submit_if_dirty(&mut self) -> Result<()> {
        if self.dirty {
            self.submit()?;
        }
        Ok(())
    }

    pub(crate) fn submit_and_wait(&mut self, n: usize) -> Result<()> {
        let errno = unsafe { __liburing_submit_and_wait(&raw mut self.ring, n as u32) };
        checkerr(errno)?;
        self.dirty = false;
        Ok(())
    }

    #[expect(dead_code)]
    pub(crate) fn wait_cqe(&mut self) -> Result<Cqe> {
        let mut cqe: *mut io_uring_cqe = core::ptr::null_mut();
        let errno = unsafe { __liburing_wait_cqe(&raw mut self.ring, &raw mut cqe) };
        checkerr(errno)?;
        if cqe.is_null() {
            bail!("got NULL from io_uring_wait_cqe");
        }
        Ok(Cqe { cqe })
    }

    pub(crate) fn try_get_cqe(&mut self) -> Result<Option<Cqe>> {
        let mut cqe: *mut io_uring_cqe = core::ptr::null_mut();
        let errno = unsafe {
            __liburing_wait_cqe_timeout(&raw mut self.ring, &raw mut cqe, &raw mut NOTIMEOUT)
        };
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
        unsafe { __liburing_cqe_seen(&raw mut self.ring, cqe.cqe) }
    }

    pub(crate) const fn fd(&self) -> i32 {
        self.ring.ring_fd
    }

    pub(crate) fn deinit(&mut self) {
        unsafe { __liburing_queue_exit(&raw mut self.ring) };
    }

    pub(crate) fn schedule(&mut self, module_id: ModuleId, wants: Wants) -> Result<()> {
        match wants {
            Wants::Socket { domain, r#type, .. } => {
                let mut sqe = self.get_sqe()?;
                sqe.prep_socket(
                    i32::from(domain.as_raw()),
                    i32::try_from(r#type.as_raw()).unwrap_or_else(|_| unreachable!()),
                    0,
                    0,
                );
                sqe.set_user_data(UserData::new(module_id, Op::Socket));
            }
            Wants::Connect { fd, addr, .. } => {
                let mut sqe = self.get_sqe()?;
                let addr =
                    if let Some(cached) = self.socket_addr_cache.iter().find(|e| ***e == addr) {
                        cached
                    } else {
                        self.socket_addr_cache.push_mut(Box::new(addr))
                    };

                sqe.prep_connect(fd, addr.as_ptr().cast(), addr.addr_len());
                sqe.set_user_data(UserData::new(module_id, Op::Connect));
            }
            Wants::Read { fd, buf, len, .. } => {
                let mut sqe = self.get_sqe()?;
                sqe.prep_read(fd, buf, len);
                sqe.set_user_data(UserData::new(module_id, Op::Read));
            }
            Wants::Write { fd, buf, len, .. } => {
                let mut sqe = self.get_sqe()?;
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
                let mut sqe = self.get_sqe()?;
                sqe.prep_read(fd, readbuf, readlen);
                sqe.set_user_data(UserData::new(module_id, Op::Read));

                let mut sqe = self.get_sqe()?;
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
                let mut sqe = self.get_sqe()?;
                sqe.prep_openat(
                    dfd,
                    path.as_ptr(),
                    i32::try_from(flags.bits()).unwrap_or_else(|_| unreachable!()),
                    mode.bits(),
                );
                sqe.set_user_data(UserData::new(module_id, Op::OpenAt));
            }
            Wants::Close { fd, .. } => {
                let mut sqe = self.get_sqe()?;
                sqe.prep_close(fd);
                sqe.set_user_data(UserData::new(module_id, Op::Close));
            }
        }

        Ok(())
    }
}
