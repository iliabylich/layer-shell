use core::ffi::CStr;
use rustix::{
    fs::{Mode, OFlags},
    net::{AddressFamily, SocketAddrAny, SocketType},
};
use std::os::fd::BorrowedFd;

#[derive(Debug)]
pub(crate) enum Wants {
    Socket {
        domain: AddressFamily,
        r#type: SocketType,
    },
    Connect {
        fd: BorrowedFd<'static>,
        addr: SocketAddrAny,
    },
    Read {
        fd: BorrowedFd<'static>,
        buf: *mut u8,
        len: usize,
    },
    Write {
        fd: BorrowedFd<'static>,
        buf: *const u8,
        len: usize,
    },
    ReadWrite {
        fd: BorrowedFd<'static>,
        readbuf: *mut u8,
        readlen: usize,
        writebuf: *const u8,
        writelen: usize,
    },
    OpenAt {
        dfd: BorrowedFd<'static>,
        path: &'static CStr,
        flags: OFlags,
        mode: Mode,
    },
    Close {
        fd: BorrowedFd<'static>,
    },
}
