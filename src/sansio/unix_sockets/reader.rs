use crate::sansio::{Satisfy, Wants};
use anyhow::{Result, bail, ensure};
use core::mem::size_of;
use libc::{AF_UNIX, SOCK_STREAM, sockaddr_un};

#[derive(Debug, Clone, Copy)]
enum State {
    ReadyToSocket,
    WaitingForSocket,

    ReadyToConnect { fd: i32 },
    WaitingForConnect { fd: i32 },

    ReadyToRead { fd: i32 },
    WaitingForRead { fd: i32 },
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
        match self.state {
            State::ReadyToSocket => {
                self.state = State::WaitingForSocket;
                Some(Wants::Socket {
                    domain: AF_UNIX,
                    type_: SOCK_STREAM,
                })
            }

            State::ReadyToConnect { fd } => {
                self.state = State::WaitingForConnect { fd };
                Some(Wants::Connect {
                    fd,
                    addr: core::ptr::from_ref(addr).cast(),
                    addrlen: size_of::<sockaddr_un>() as u32,
                })
            }

            State::ReadyToRead { fd } => {
                self.state = State::WaitingForRead { fd };
                Some(Wants::Read {
                    fd,
                    buf: self.buf.as_mut_ptr(),
                    len: self.buf.len(),
                })
            }

            _ => None,
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy) -> Result<Option<([u8; 1_024], usize)>> {
        match (self.state, satisfy) {
            (State::WaitingForSocket, Satisfy::Socket(res)) => {
                let fd = res?;
                self.state = State::ReadyToConnect { fd };
                Ok(None)
            }

            (State::WaitingForConnect { fd }, Satisfy::Connect(res)) => {
                res?;
                self.state = State::ReadyToRead { fd };
                Ok(None)
            }

            (State::WaitingForRead { fd }, Satisfy::Read(res)) => {
                let bytes_read = res?;
                ensure!(bytes_read != 0, "EOF");
                let buf = self.buf;
                self.buf = [0; _];
                self.state = State::ReadyToRead { fd };

                Ok(Some((buf, bytes_read)))
            }

            (state, satisfy) => {
                bail!("malformed state: {state:?} vs {satisfy:?}")
            }
        }
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
