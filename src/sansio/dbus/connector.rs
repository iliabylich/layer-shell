use crate::sansio::{Satisfy, Wants};
use anyhow::{Result, bail, ensure};
use libc::{AF_UNIX, SOCK_STREAM, sockaddr, sockaddr_un};

#[derive(Debug, Clone, Copy)]
enum State {
    CanSocket,
    WaitingForSocket,
    CanConnect,
    WaitingForConnect,
    CanWriteZero,
    WaitingForWriteZero,
    CanWriteAuthExternal,
    WaitingForWriteAuthExternal,
    CanReadData,
    WaitingForReadData,
    CanWriteData,
    WaitingForWriteData,
    CanReadGUID,
    WaitingForReadGUID,
    CanWriteBegin,
    WaitingForWriteBegin,
    Done,
}

pub(crate) struct DBusConnector {
    fd: i32,
    state: State,
    addr: sockaddr_un,
    buf: [u8; 100],
}

impl DBusConnector {
    pub(crate) fn new(addr: sockaddr_un) -> Self {
        Self {
            fd: -1,
            state: State::CanSocket,
            addr,
            buf: [0; _],
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
                    addrlen: std::mem::size_of::<sockaddr_un>() as u32,
                }
            }
            State::WaitingForConnect => Wants::Nothing,

            State::CanWriteZero => {
                self.state = State::WaitingForWriteZero;
                let buf = b"\0";
                Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                }
            }
            State::WaitingForWriteZero => Wants::Nothing,

            State::CanWriteAuthExternal => {
                self.state = State::WaitingForWriteAuthExternal;
                let buf = b"AUTH EXTERNAL\r\n";
                Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                }
            }
            State::WaitingForWriteAuthExternal => Wants::Nothing,

            State::CanReadData => {
                self.state = State::WaitingForReadData;
                Wants::Read {
                    fd: self.fd,
                    buf: self.buf.as_mut_ptr(),
                    len: self.buf.len(),
                }
            }
            State::WaitingForReadData => Wants::Nothing,

            State::CanWriteData => {
                self.state = State::WaitingForWriteData;
                let buf = b"DATA\r\n";
                Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                }
            }
            State::WaitingForWriteData => Wants::Nothing,

            State::CanReadGUID => {
                self.state = State::WaitingForReadGUID;
                Wants::Read {
                    fd: self.fd,
                    buf: self.buf.as_mut_ptr(),
                    len: self.buf.len(),
                }
            }
            State::WaitingForReadGUID => Wants::Nothing,

            State::CanWriteBegin => {
                self.state = State::WaitingForWriteBegin;
                let buf = b"BEGIN\r\n";
                Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                }
            }
            State::WaitingForWriteBegin => Wants::Nothing,

            State::Done => Wants::Nothing,
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<i32>> {
        match (self.state, satisfy) {
            (State::WaitingForSocket, Satisfy::Socket) => {
                ensure!(res >= 0);
                self.fd = res;
                self.state = State::CanConnect;
                Ok(None)
            }

            (State::WaitingForConnect, Satisfy::Connect) => {
                ensure!(res >= 0);
                self.state = State::CanWriteZero;
                Ok(None)
            }

            (State::WaitingForWriteZero, Satisfy::Write) => {
                ensure!(res >= 0);
                let bytes_written = res as usize;
                ensure!(bytes_written == b"\0".len());
                self.state = State::CanWriteAuthExternal;
                Ok(None)
            }

            (State::WaitingForWriteAuthExternal, Satisfy::Write) => {
                ensure!(res >= 0);
                let bytes_written = res as usize;
                ensure!(bytes_written == b"AUTH EXTERNAL\r\n".len());
                self.state = State::CanReadData;
                Ok(None)
            }

            (State::WaitingForReadData, Satisfy::Read) => {
                ensure!(res >= 0);
                let bytes_read = res as usize;
                ensure!(&self.buf[..bytes_read] == b"DATA\r\n");
                self.state = State::CanWriteData;
                Ok(None)
            }

            (State::WaitingForWriteData, Satisfy::Write) => {
                ensure!(res >= 0);
                let bytes_written = res as usize;
                ensure!(bytes_written == b"DATA\r\n".len());
                self.state = State::CanReadGUID;
                Ok(None)
            }

            (State::WaitingForReadGUID, Satisfy::Read) => {
                ensure!(res > 0);
                self.state = State::CanWriteBegin;
                Ok(None)
            }

            (State::WaitingForWriteBegin, Satisfy::Write) => {
                ensure!(res >= 0);
                let bytes_written = res as usize;
                ensure!(bytes_written == b"BEGIN\r\n".len());
                self.state = State::Done;
                Ok(Some(self.fd))
            }

            (state, satisfy) => {
                bail!("malformed DBusConnector state: {state:?} vs {satisfy:?}")
            }
        }
    }
}
