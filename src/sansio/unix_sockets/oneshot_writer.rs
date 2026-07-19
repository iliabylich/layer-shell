use crate::sansio::{Satisfy, Wants};
use anyhow::{Result, bail};
use libc::{AF_UNIX, SOCK_STREAM, sockaddr_un};

pub(crate) struct UnixSocketOneshotWriter {
    data: &'static [u8],
    offset: usize,
    state: State,
}

#[derive(Debug, Clone, Copy)]
enum State {
    ReadyToSocket,
    WaitingForSocket,

    ReadyToConnect { fd: i32 },
    WaitingForConnect { fd: i32 },

    ReadyToWrite { fd: i32 },
    WaitingForWrite { fd: i32 },
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
                    addrlen: size_of::<sockaddr_un>() as u32,
                })
            }

            State::ReadyToWrite { fd } => {
                self.state = State::WaitingForWrite { fd };
                let buf = self
                    .data
                    .get(self.offset..)
                    .unwrap_or_else(|| unreachable!());

                Some(Wants::Write {
                    fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                })
            }

            _ => None,
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy) -> Result<Option<i32>> {
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

            (state, satisfy) => {
                bail!("malformed state: {state:?} vs {satisfy:?}")
            }
        }
    }
}
