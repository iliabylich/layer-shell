use crate::{
    error::IoError,
    sansio::{Satisfy, Wants},
    utils::log_err_and_exit,
};
use libc::{AF_UNIX, SOCK_STREAM, sockaddr_un};
use rustix::fd::BorrowedFd;

#[derive(Debug, Clone, Copy)]
pub struct UnixSocketOneshotWriter {
    data: &'static [u8],
    offset: usize,
    state: State,
}

#[derive(Debug, Clone, Copy)]
enum State {
    ReadyToSocket,
    WaitingForSocket,

    ReadyToConnect { fd: BorrowedFd<'static> },
    WaitingForConnect { fd: BorrowedFd<'static> },

    ReadyToWrite { fd: BorrowedFd<'static> },
    WaitingForWrite { fd: BorrowedFd<'static> },
}
impl State {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ReadyToSocket => "ReadyToSocket",
            Self::WaitingForSocket => "WaitingForSocket",
            Self::ReadyToConnect { .. } => "ReadyToConnect",
            Self::WaitingForConnect { .. } => "WaitingForConnect",
            Self::ReadyToWrite { .. } => "ReadyToWrite",
            Self::WaitingForWrite { .. } => "WaitingForWrite",
        }
    }
}

impl UnixSocketOneshotWriter {
    pub(crate) const fn new(data: &'static [u8]) -> Self {
        Self {
            data,
            offset: 0,
            state: State::ReadyToSocket,
        }
    }

    pub(crate) fn wants(&mut self, addr: &sockaddr_un) -> Option<Wants> {
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
                    addrlen: u32::try_from(size_of::<sockaddr_un>()).unwrap_or_else(|_| {
                        log_err_and_exit!("sockaddr_un size doesn't fit into u32")
                    }),
                })
            }

            State::ReadyToWrite { fd } => {
                self.state = State::WaitingForWrite { fd };
                let buf = self
                    .data
                    .get(self.offset..)
                    .unwrap_or_else(|| log_err_and_exit!("malformed state"));

                Some(Wants::Write {
                    fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                })
            }

            _ => None,
        }
    }

    pub(crate) fn satisfy(
        &mut self,
        satisfy: Satisfy,
    ) -> Result<Option<BorrowedFd<'static>>, IoError> {
        match (self.state, satisfy) {
            (State::WaitingForSocket, Satisfy::Socket(res)) => {
                let fd = res?;
                self.state = State::ReadyToConnect { fd };
                Ok(None)
            }

            (State::WaitingForConnect { fd }, Satisfy::Connect(res)) => {
                res?;
                self.state = State::ReadyToWrite { fd };
                Ok(None)
            }

            (State::WaitingForWrite { fd }, Satisfy::Write(res)) => {
                self.offset += res?;
                if self.offset == self.data.len() {
                    Ok(Some(fd))
                } else {
                    Ok(None)
                }
            }

            _ => Err(IoError::WrongSatisfy {
                state: self.state.as_str(),
                satisfy: satisfy.as_str(),
            }),
        }
    }
}
