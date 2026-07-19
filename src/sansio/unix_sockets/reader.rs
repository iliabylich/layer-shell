use crate::sansio::{Satisfy, Wants};
use anyhow::{Result, bail, ensure};
use core::mem::size_of;
use libc::{AF_UNIX, SOCK_STREAM, sockaddr_un};

#[derive(Debug, Clone, Copy)]
pub(crate) enum UnixSocketReader {
    ReadyToSocket,
    WaitingForSocket,

    ReadyToConnect { fd: i32 },
    WaitingForConnect { fd: i32 },

    ReadyToRead { fd: i32 },
    WaitingForRead { fd: i32 },
}

impl UnixSocketReader {
    pub(crate) const fn new() -> Self {
        Self::ReadyToSocket
    }

    pub(crate) const fn new_connected_from_fd(fd: i32) -> Self {
        Self::ReadyToRead { fd }
    }

    pub(crate) const fn wants(&mut self, addr: &sockaddr_un, buf: &mut [u8]) -> Option<Wants> {
        match *self {
            Self::ReadyToSocket => {
                *self = Self::WaitingForSocket;
                Some(Wants::Socket {
                    domain: AF_UNIX,
                    type_: SOCK_STREAM,
                })
            }

            Self::ReadyToConnect { fd } => {
                *self = Self::WaitingForConnect { fd };
                Some(Wants::Connect {
                    fd,
                    addr: core::ptr::from_ref(addr).cast(),
                    addrlen: size_of::<sockaddr_un>() as u32,
                })
            }

            Self::ReadyToRead { fd } => {
                *self = Self::WaitingForRead { fd };
                Some(Wants::Read {
                    fd,
                    buf: buf.as_mut_ptr(),
                    len: buf.len(),
                })
            }

            _ => None,
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy) -> Result<Option<usize>> {
        match (*self, satisfy) {
            (Self::WaitingForSocket, Satisfy::Socket(res)) => {
                let fd = res?;
                *self = Self::ReadyToConnect { fd };
                Ok(None)
            }

            (Self::WaitingForConnect { fd }, Satisfy::Connect(res)) => {
                res?;
                *self = Self::ReadyToRead { fd };
                Ok(None)
            }

            (Self::WaitingForRead { fd }, Satisfy::Read(res)) => {
                let bytes_read = res?;
                ensure!(bytes_read != 0, "EOF");
                *self = Self::ReadyToRead { fd };

                Ok(Some(bytes_read))
            }

            (state, satisfy) => {
                bail!("malformed state: {state:?} vs {satisfy:?}")
            }
        }
    }

    pub(crate) const fn fd(self) -> Option<i32> {
        match self {
            Self::ReadyToSocket | Self::WaitingForSocket => None,

            Self::ReadyToConnect { fd }
            | Self::WaitingForConnect { fd }
            | Self::ReadyToRead { fd }
            | Self::WaitingForRead { fd } => Some(fd),
        }
    }
}
