use crate::{
    external::{
        __liburing_prep_accept, __liburing_prep_connect, __liburing_prep_read,
        __liburing_prep_socket, __liburing_prep_write, io_uring_sqe, sockaddr,
    },
    user_data::UserData,
};
use core::ffi::c_void;

#[derive(Clone, Copy)]
pub struct Sqe {
    pub(crate) sqe: *mut io_uring_sqe,
}

impl Sqe {
    pub(crate) fn prep_socket(&mut self, domain: i32, type_: i32, protocol: i32, flags: u32) {
        unsafe { __liburing_prep_socket(self.sqe, domain, type_, protocol, flags) }
    }
    pub(crate) fn prep_connect(&mut self, fd: i32, addr: *const sockaddr, addrlen: u32) {
        unsafe { __liburing_prep_connect(self.sqe, fd, addr.cast(), addrlen) }
    }
    pub(crate) fn prep_write(&mut self, fd: i32, ptr: *const c_void, len: u32) {
        unsafe { __liburing_prep_write(self.sqe, fd, ptr.cast(), len, 0) }
    }
    pub(crate) fn prep_read(&mut self, fd: i32, ptr: *mut c_void, len: u32) {
        unsafe { __liburing_prep_read(self.sqe, fd, ptr, len, 0) }
    }
    pub(crate) fn prep_accept(
        &mut self,
        fd: i32,
        addr: *mut sockaddr,
        addrlen: *mut u32,
        flags: i32,
    ) {
        unsafe { __liburing_prep_accept(self.sqe, fd, addr, addrlen, flags) }
    }

    pub(crate) fn set_user_data(&mut self, data: UserData) {
        (unsafe { &mut *self.sqe }).user_data = data.encode();
    }
}
