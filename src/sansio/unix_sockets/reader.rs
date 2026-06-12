use crate::sansio::Wants;
use anyhow::{Result, bail, ensure};
use rustix::net::{AddressFamily, SocketAddrUnix, SocketType};
use std::os::fd::BorrowedFd;

#[derive(Debug)]
enum State {
    ReadyToSocket {
        addr: SocketAddrUnix,
    },
    WaitingForSocket {
        addr: SocketAddrUnix,
    },

    ReadyToConnect {
        addr: SocketAddrUnix,
        fd: BorrowedFd<'static>,
    },
    WaitingForConnect {
        fd: BorrowedFd<'static>,
    },

    ReadyToRead {
        fd: BorrowedFd<'static>,
    },
    WaitingForRead {
        fd: BorrowedFd<'static>,
    },

    Disconnected,
}

impl State {
    fn wants(self, seq: u64, buf: &mut [u8]) -> (Self, Option<Wants>) {
        match self {
            Self::ReadyToSocket { addr } => (
                Self::WaitingForSocket { addr },
                Some(Wants::Socket {
                    domain: AddressFamily::UNIX,
                    r#type: SocketType::STREAM,
                    seq,
                }),
            ),

            Self::ReadyToConnect { addr, fd } => (
                Self::WaitingForConnect { fd },
                Some(Wants::Connect {
                    fd,
                    addr: addr.into(),
                    seq,
                }),
            ),

            Self::ReadyToRead { fd } => (
                Self::WaitingForRead { fd },
                Some(Wants::Read {
                    fd,
                    buf: buf.as_mut_ptr(),
                    len: buf.len(),
                    seq,
                }),
            ),

            waiting => (waiting, None),
        }
    }

    fn wants_in_place(&mut self, seq: u64, buf: &mut [u8]) -> Option<Wants> {
        let mut this = Self::Disconnected;
        std::mem::swap(self, &mut this);
        let (next, wants) = this.wants(seq, buf);
        *self = next;
        wants
    }
}

pub(crate) struct UnixSocketReader {
    buf: [u8; 1_024],
    state: State,
    seq: u64,
}

impl UnixSocketReader {
    pub(crate) const fn new(addr: SocketAddrUnix) -> Self {
        Self {
            buf: [0; _],
            state: State::ReadyToSocket { addr },
            seq: 0,
        }
    }

    pub(crate) const fn new_connected_from_fd(fd: BorrowedFd<'static>) -> Self {
        Self {
            buf: [0; _],
            state: State::ReadyToRead { fd },
            seq: 0,
        }
    }

    pub(crate) const fn dummy() -> Self {
        Self {
            buf: [0; _],
            state: State::Disconnected,
            seq: 0,
        }
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        self.state.wants_in_place(self.seq, &mut self.buf)
    }

    pub(crate) fn satisfy_socket(&mut self, fd: BorrowedFd<'static>) -> Result<()> {
        let State::WaitingForSocket { addr } = &self.state else {
            bail!("malformed state: expected Socket, got {:?}", self.state);
        };

        self.state = State::ReadyToConnect {
            addr: addr.clone(),
            fd,
        };
        self.seq += 1;
        Ok(())
    }

    pub(crate) fn satisfy_connect(&mut self) -> Result<()> {
        let State::WaitingForConnect { fd, .. } = &self.state else {
            bail!("malformed state: expected Connect, got {:?}", self.state);
        };

        self.state = State::ReadyToRead { fd: *fd };
        self.seq += 1;
        Ok(())
    }

    pub(crate) fn satisfy_read(&mut self, bytes_read: usize) -> Result<([u8; 1_024], usize)> {
        let State::WaitingForRead { fd } = &self.state else {
            bail!("malformed state: expected Read, got {:?}", self.state);
        };

        ensure!(bytes_read != 0, "EOF");
        let buf = self.buf;
        self.buf = [0; _];
        self.seq += 1;
        self.state = State::ReadyToRead { fd: *fd };

        Ok((buf, bytes_read))
    }
}
