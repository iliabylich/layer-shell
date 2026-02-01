use super::generated::{
    __liburing_prep_close, __liburing_prep_connect, __liburing_prep_openat, __liburing_prep_read,
    __liburing_prep_socket, __liburing_prep_write, io_uring_sqe, mode_t, socklen_t,
};
use libc::sockaddr;

#[derive(Clone, Copy)]
pub(crate) struct Sqe {
    pub(crate) sqe: *mut io_uring_sqe,
}

impl Sqe {
    pub(crate) fn prep_socket(&mut self, domain: i32, type_: i32, protocol: i32, flags: u32) {
        unsafe { __liburing_prep_socket(self.sqe, domain, type_, protocol, flags) }
    }
    pub(crate) fn prep_connect(&mut self, fd: i32, addr: *const sockaddr, addrlen: socklen_t) {
        unsafe { __liburing_prep_connect(self.sqe, fd, addr.cast(), addrlen) }
    }
    pub(crate) fn prep_write(&mut self, fd: i32, ptr: *const u8, len: usize) {
        unsafe { __liburing_prep_write(self.sqe, fd, ptr.cast(), len as u32, 0) }
    }
    pub(crate) fn prep_read(&mut self, fd: i32, ptr: *mut u8, len: usize) {
        unsafe { __liburing_prep_read(self.sqe, fd, ptr.cast(), len as u32, 0) }
    }
    pub(crate) fn prep_close(&mut self, fd: i32) {
        unsafe { __liburing_prep_close(self.sqe, fd) }
    }
    #[allow(dead_code)]
    pub(crate) fn prep_openat(
        &mut self,
        dfd: i32,
        path: *const ::std::os::raw::c_char,
        flags: i32,
        mode: mode_t,
    ) {
        unsafe { __liburing_prep_openat(self.sqe, dfd, path, flags, mode) }
    }

    pub(crate) fn set_user_data(&mut self, data: impl Into<u64>) {
        (unsafe { self.sqe.as_mut().unwrap() }).user_data = data.into();
    }
}
