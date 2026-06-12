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
        seq: u64,
    },
    Connect {
        fd: BorrowedFd<'static>,
        addr: SocketAddrAny,
        seq: u64,
    },
    Read {
        fd: BorrowedFd<'static>,
        buf: *mut u8,
        len: usize,
        seq: u64,
    },
    Write {
        fd: BorrowedFd<'static>,
        buf: *const u8,
        len: usize,
        seq: u64,
    },
    ReadWrite {
        fd: BorrowedFd<'static>,
        readbuf: *mut u8,
        readlen: usize,
        readseq: u64,
        writebuf: *const u8,
        writelen: usize,
        writeseq: u64,
    },
    OpenAt {
        dfd: BorrowedFd<'static>,
        path: &'static str,
        flags: OFlags,
        mode: Mode,
        seq: u64,
    },
    Close {
        fd: BorrowedFd<'static>,
        seq: u64,
    },
}
