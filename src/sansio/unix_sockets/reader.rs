use crate::sansio::{Satisfy, Wants};
use anyhow::{Result, bail, ensure};
use libc::{AF_UNIX, SOCK_STREAM, sockaddr, sockaddr_un};

#[derive(Debug, Clone, Copy)]
enum State {
    CanSocket,
    WaitingForSocket,
    CanConnect,
    WaitingForConnect,
    CanRead,
    WaitingForRead,
}

pub(crate) struct UnixSocketReader {
    addr: sockaddr_un,
    fd: i32,
    buf: [u8; 1_024],
    state: State,
}

impl UnixSocketReader {
    pub(crate) fn new(addr: sockaddr_un) -> Self {
        Self {
            addr,
            fd: -1,
            buf: [0; _],
            state: State::CanSocket,
        }
    }

    pub(crate) fn wants(&mut self) -> Wants {
        match self.state {
            State::CanSocket => {
                self.state = State::WaitingForSocket;
                Wants::Socket {
                    domain: AF_UNIX,
                    r#type: SOCK_STREAM,
                }
            }
            State::WaitingForSocket => Wants::Nothing,

            State::CanConnect => {
                self.state = State::WaitingForConnect;
                Wants::Connect {
                    fd: self.fd,
                    addr: (&self.addr as *const sockaddr_un).cast::<sockaddr>(),
                    addrlen: core::mem::size_of::<sockaddr_un>() as u32,
                }
            }
            State::WaitingForConnect => Wants::Nothing,

            State::CanRead => {
                self.state = State::WaitingForRead;
                Wants::Read {
                    fd: self.fd,
                    buf: self.buf.as_mut_ptr(),
                    len: self.buf.len(),
                }
            }
            State::WaitingForRead => Wants::Nothing,
        }
    }

    pub(crate) fn satisfy(
        &mut self,
        satisfy: Satisfy,
        res: i32,
    ) -> Result<Option<([u8; 1_024], usize)>> {
        match (self.state, satisfy) {
            (State::WaitingForSocket, Satisfy::Socket) => {
                ensure!(res >= 0, "UnixSocketReader::Socket failed: {res}");
                self.fd = res;
                self.state = State::CanConnect;
                Ok(None)
            }

            (State::WaitingForConnect, Satisfy::Connect) => {
                ensure!(res >= 0, "UnixSocketReader::Connect failed: {res}");
                self.state = State::CanRead;
                Ok(None)
            }

            (State::WaitingForRead, Satisfy::Read) => {
                ensure!(res > 0, "UnixSocketReader::Read failed: {res}");
                let len = res as usize;
                let buf = self.buf;
                self.buf = [0; _];
                self.state = State::CanRead;
                Ok(Some((buf, len)))
            }

            (state, satisfy) => {
                bail!("malformed UnixSocketReader state: {state:?} vs {satisfy:?}")
            }
        }
    }
}
