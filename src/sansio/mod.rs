mod dns;
mod https;
mod timerfd;
mod tls_over_tcp;
mod unix_sockets;

pub(crate) use dns::Dns;
pub(crate) use https::{Https, HttpsRequest, HttpsResponse};
pub(crate) use timerfd::TimerFd;
pub(crate) use tls_over_tcp::TlsOverTcp;
pub(crate) use unix_sockets::UnixSocketOneshotWriter;

use crate::macros::assert_or_exit;

#[derive(Debug)]
pub(crate) enum Wants<'a> {
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
        buf: &'a mut [u8],
    },
    Write {
        fd: i32,
        buf: &'a [u8],
    },
    Close {
        fd: i32,
    },

    Nothing,
}

#[derive(Debug)]

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
