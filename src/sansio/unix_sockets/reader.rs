use crate::{
    error::IoError,
    sansio::{Satisfy, Wants},
    utils::log_err_and_exit,
};
use core::mem::size_of;
use libc::sockaddr_un;
use rustix::{
    fd::BorrowedFd,
    net::{AddressFamily, SocketType},
};

#[derive(Debug, Clone, Copy)]
pub enum UnixSocketReader {
    ReadyToSocket,
    WaitingForSocket,

    ReadyToConnect { fd: BorrowedFd<'static> },
    WaitingForConnect { fd: BorrowedFd<'static> },

    ReadyToRead { fd: BorrowedFd<'static> },
    WaitingForRead { fd: BorrowedFd<'static> },
}

impl UnixSocketReader {
    pub(crate) const fn new() -> Self {
        Self::ReadyToSocket
    }

    pub(crate) const fn new_connected_from_fd(fd: BorrowedFd<'static>) -> Self {
        Self::ReadyToRead { fd }
    }

    pub(crate) fn wants(&mut self, addr: &sockaddr_un, buf: &mut [u8]) -> Option<Wants> {
        match *self {
            Self::ReadyToSocket => {
                *self = Self::WaitingForSocket;
                Some(Wants::Socket {
                    domain: AddressFamily::UNIX,
                    type_: SocketType::STREAM,
                })
            }

            Self::ReadyToConnect { fd } => {
                *self = Self::WaitingForConnect { fd };
                Some(Wants::Connect {
                    fd,
                    addr: core::ptr::from_ref(addr).cast(),
                    addrlen: u32::try_from(size_of::<sockaddr_un>()).unwrap_or_else(|_| {
                        log_err_and_exit!("sockadd_un size doesn't fit into u32")
                    }),
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

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy) -> Result<Option<usize>, IoError> {
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
                if bytes_read == 0 {
                    return Err(IoError::EofError);
                }
                *self = Self::ReadyToRead { fd };

                Ok(Some(bytes_read))
            }

            _ => Err(IoError::WrongSatisfy {
                state: self.as_str(),
                satisfy: satisfy.as_str(),
            }),
        }
    }

    pub(crate) const fn fd(self) -> Option<BorrowedFd<'static>> {
        match self {
            Self::ReadyToSocket | Self::WaitingForSocket => None,

            Self::ReadyToConnect { fd }
            | Self::WaitingForConnect { fd }
            | Self::ReadyToRead { fd }
            | Self::WaitingForRead { fd } => Some(fd),
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::ReadyToSocket => "ReadyToSocket",
            Self::WaitingForSocket => "WaitingForSocket",
            Self::ReadyToConnect { .. } => "ReadyToConnect",
            Self::WaitingForConnect { .. } => "WaitingForConnect",
            Self::ReadyToRead { .. } => "ReadyToRead",
            Self::WaitingForRead { .. } => "WaitingForRead",
        }
    }
}
