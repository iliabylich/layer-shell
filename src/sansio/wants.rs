use core::ffi::CStr;
use libc::sockaddr;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Wants {
    Socket {
        domain: i32,
        type_: i32,
    },
    Connect {
        fd: i32,
        addr: *const sockaddr,
        addrlen: u32,
    },
    Read {
        fd: i32,
        buf: *mut u8,
        len: usize,
    },
    Write {
        fd: i32,
        buf: *const u8,
        len: usize,
    },
    OpenAt {
        dfd: i32,
        path: &'static CStr,
        flags: i32,
        mode: u32,
    },
    Close {
        fd: i32,
    },
    Accept {
        fd: i32,
    },
}
