use core::ffi::CStr;
use libc::{sockaddr, socklen_t};

#[derive(Debug, Clone, Copy)]
pub(crate) enum Wants {
    Socket {
        domain: i32,
        type_: i32,
    },
    Connect {
        fd: i32,
        addr: *const sockaddr,
        addrlen: socklen_t,
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
    ReadWrite {
        fd: i32,
        readbuf: *mut u8,
        readlen: usize,
        writebuf: *const u8,
        writelen: usize,
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
}
