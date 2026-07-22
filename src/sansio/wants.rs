use crate::external::sockaddr;
use rustix::{
    fd::BorrowedFd,
    net::{AddressFamily, SocketType},
};

#[derive(Debug, Clone, Copy)]
pub enum Wants {
    Socket {
        domain: AddressFamily,
        type_: SocketType,
    },
    Connect {
        fd: BorrowedFd<'static>,
        addr: *const sockaddr,
        addrlen: u32,
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
    Accept {
        fd: BorrowedFd<'static>,
    },
}
