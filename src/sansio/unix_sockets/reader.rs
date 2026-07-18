use crate::sansio::Wants;
use anyhow::{Result, bail, ensure};
use core::mem::size_of;
use libc::{AF_UNIX, SOCK_STREAM, sockaddr_un};

#[derive(Debug)]
enum State {
    ReadyToSocket,
    WaitingForSocket,

    ReadyToConnect { fd: i32 },
    WaitingForConnect { fd: i32 },

    ReadyToRead { fd: i32 },
    WaitingForRead { fd: i32 },
}

impl State {
    const fn wants(self, buf: &mut [u8], addr: &sockaddr_un) -> (Self, Option<Wants>) {
        match self {
            Self::ReadyToSocket => (
                Self::WaitingForSocket,
                Some(Wants::Socket {
                    domain: AF_UNIX,
                    type_: SOCK_STREAM,
                }),
            ),

            Self::ReadyToConnect { fd } => (
                Self::WaitingForConnect { fd },
                Some(Wants::Connect {
                    fd,
                    addr: core::ptr::from_ref(addr).cast(),
                    addrlen: size_of::<sockaddr_un>() as u32,
                }),
            ),

            Self::ReadyToRead { fd } => (
                Self::WaitingForRead { fd },
                Some(Wants::Read {
                    fd,
                    buf: buf.as_mut_ptr(),
                    len: buf.len(),
                }),
            ),

            waiting => (waiting, None),
        }
    }

    const fn wants_in_place(&mut self, buf: &mut [u8], addr: &sockaddr_un) -> Option<Wants> {
        let mut this: Self = unsafe { core::mem::zeroed() };
        core::mem::swap(self, &mut this);
        let (next, wants) = this.wants(buf, addr);
        *self = next;
        wants
    }
}

pub(crate) struct UnixSocketReader {
    buf: [u8; 1_024],
    state: State,
}

impl UnixSocketReader {
    pub(crate) const fn new() -> Self {
        Self {
            buf: [0; _],
            state: State::ReadyToSocket,
        }
    }

    pub(crate) const fn new_connected_from_fd(fd: i32) -> Self {
        Self {
            buf: [0; _],
            state: State::ReadyToRead { fd },
        }
    }

    pub(crate) const fn wants(&mut self, addr: &sockaddr_un) -> Option<Wants> {
        self.state.wants_in_place(&mut self.buf, addr)
    }

    pub(crate) fn satisfy_socket(&mut self, fd: i32) -> Result<()> {
        let State::WaitingForSocket = &self.state else {
            bail!("malformed state: expected Socket, got {:?}", self.state);
        };

        self.state = State::ReadyToConnect { fd };
        Ok(())
    }

    pub(crate) fn satisfy_connect(&mut self) -> Result<()> {
        let State::WaitingForConnect { fd, .. } = &self.state else {
            bail!("malformed state: expected Connect, got {:?}", self.state);
        };

        self.state = State::ReadyToRead { fd: *fd };
        Ok(())
    }

    pub(crate) fn satisfy_read(&mut self, bytes_read: usize) -> Result<([u8; 1_024], usize)> {
        let State::WaitingForRead { fd } = &self.state else {
            bail!("malformed state: expected Read, got {:?}", self.state);
        };

        ensure!(bytes_read != 0, "EOF");
        let buf = self.buf;
        self.buf = [0; _];
        self.state = State::ReadyToRead { fd: *fd };

        Ok((buf, bytes_read))
    }

    pub(crate) const fn fd(&self) -> Option<i32> {
        match &self.state {
            State::ReadyToSocket | State::WaitingForSocket => None,

            State::ReadyToConnect { fd }
            | State::WaitingForConnect { fd }
            | State::ReadyToRead { fd }
            | State::WaitingForRead { fd } => Some(*fd),
        }
    }
}
