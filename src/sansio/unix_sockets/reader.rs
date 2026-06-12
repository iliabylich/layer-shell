use crate::sansio::Wants;
use anyhow::{Result, ensure};
use libc::{AF_UNIX, SOCK_STREAM, sockaddr, sockaddr_un};
use std::os::fd::{AsRawFd, BorrowedFd};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Socket,
    Connect,
    Read,
}

pub(crate) struct UnixSocketReader {
    addr: sockaddr_un,
    fd: i32,
    buf: [u8; 1_024],
    state: State,
    seq: u64,
}

impl UnixSocketReader {
    pub(crate) const fn new(addr: sockaddr_un) -> Self {
        Self {
            addr,
            fd: -1,
            buf: [0; _],
            state: State::Socket,
            seq: 0,
        }
    }

    pub(crate) const fn new_connected_from_fd(fd: i32) -> Self {
        Self {
            addr: unsafe { std::mem::MaybeUninit::zeroed().assume_init() },
            fd,
            buf: [0; _],
            state: State::Read,
            seq: 0,
        }
    }

    pub(crate) const fn dummy() -> Self {
        Self {
            addr: unsafe { std::mem::MaybeUninit::zeroed().assume_init() },
            fd: -1,
            buf: [0; _],
            state: State::Socket,
            seq: 0,
        }
    }

    pub(crate) const fn wants(&mut self) -> Wants {
        match self.state {
            State::Socket => Wants::Socket {
                domain: AF_UNIX,
                r#type: SOCK_STREAM,
                seq: self.seq,
            },

            State::Connect => Wants::Connect {
                fd: self.fd,
                addr: (&raw const self.addr).cast::<sockaddr>(),
                addrlen: size_of::<sockaddr_un>() as u32,
                seq: self.seq,
            },

            State::Read => Wants::Read {
                fd: self.fd,
                buf: self.buf.as_mut_ptr(),
                len: self.buf.len(),
                seq: self.seq,
            },
        }
    }

    pub(crate) fn satisfy_socket(&mut self, fd: BorrowedFd<'static>) -> Result<()> {
        ensure!(
            self.state == State::Socket,
            "malformed state: expected Socket, got {:?}",
            self.state
        );

        self.fd = fd.as_raw_fd();
        self.state = State::Connect;
        self.seq += 1;
        Ok(())
    }

    pub(crate) fn satisfy_connect(&mut self) -> Result<()> {
        ensure!(
            self.state == State::Connect,
            "malformed state: expected Connect, got {:?}",
            self.state
        );

        self.state = State::Read;
        self.seq += 1;
        Ok(())
    }

    pub(crate) fn satisfy_read(&mut self, bytes_read: usize) -> Result<([u8; 1_024], usize)> {
        ensure!(
            self.state == State::Read,
            "malformed state: expected Read, got {:?}",
            self.state
        );

        ensure!(bytes_read != 0, "EOF");
        let buf = self.buf;
        self.buf = [0; _];
        self.seq += 1;

        Ok((buf, bytes_read))
    }
}
