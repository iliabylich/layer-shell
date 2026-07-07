use core::ffi::CStr;
use rustix::{
    fs::{Mode, OFlags},
    net::{AddressFamily, SocketAddrAny, SocketType},
};

#[derive(Debug)]
pub(crate) enum Wants {
    Socket {
        domain: AddressFamily,
        r#type: SocketType,
    },
    Connect {
        fd: i32,
        addr: SocketAddrAny,
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
        flags: OFlags,
        mode: Mode,
    },
    Close {
        fd: i32,
    },
}
