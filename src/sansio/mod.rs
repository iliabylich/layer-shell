mod dbus;
mod dns;
mod https;
mod timerfd;
mod tls_over_tcp;
mod unix_sockets;

pub(crate) use dbus::DBusConnection;
pub(crate) use dns::Dns;
pub(crate) use https::{Https, HttpsRequest, HttpsResponse};
pub(crate) use timerfd::TimerFd;
pub(crate) use tls_over_tcp::TlsOverTcp;
pub(crate) use unix_sockets::{UnixSocketOneshotWriter, UnixSocketReader};

use crate::macros::assert_or_exit;

#[derive(Debug)]
pub(crate) enum Wants {
    Socket {
        domain: i32,
        r#type: i32,
    },
    Connect {
        fd: i32,
        addr: *const libc::sockaddr,
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
    ReadWrite {
        fd: i32,
        readbuf: *mut u8,
        readlen: usize,
        writebuf: *const u8,
        writelen: usize,
    },
    Close {
        fd: i32,
    },

    Nothing,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]

pub(crate) enum Satisfy {
    Socket,
    Connect,
    Write,
    Read,
    Close,
}
const MAX: Satisfy = Satisfy::Close;

impl From<Satisfy> for u8 {
    fn from(value: Satisfy) -> Self {
        value as u8
    }
}

impl From<u8> for Satisfy {
    fn from(value: u8) -> Self {
        assert_or_exit!(
            value <= MAX as u8,
            "received malformed Satisfy from io_uring: {value}"
        );
        unsafe { std::mem::transmute::<u8, Self>(value) }
    }
}
