use crate::sansio::Wants;
use anyhow::{Result, bail, ensure};
use rustix::net::{AddressFamily, SocketAddrUnix, SocketType};

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
    fn wants(self, buf: &mut [u8], addr: &SocketAddrUnix) -> (Self, Option<Wants>) {
        match self {
            Self::ReadyToSocket => (
                Self::WaitingForSocket,
                Some(Wants::Socket {
                    domain: AddressFamily::UNIX,
                    r#type: SocketType::STREAM,
                }),
            ),

            Self::ReadyToConnect { fd } => (
                Self::WaitingForConnect { fd },
                Some(Wants::Connect {
                    fd,
                    addr: addr.clone().into(),
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

    fn wants_in_place(&mut self, buf: &mut [u8], addr: &SocketAddrUnix) -> Option<Wants> {
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

    pub(crate) fn wants(&mut self, addr: &SocketAddrUnix) -> Option<Wants> {
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
}
