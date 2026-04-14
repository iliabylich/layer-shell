use crate::sansio::{DBusConnectionKind, Satisfy, Wants};
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
    Dead,
}

pub(crate) struct DBusConnector {
    fd: i32,
    state: State,
    addr: sockaddr_un,
    buf: [u8; 100],
    kind: DBusConnectionKind,
}

impl DBusConnector {
    pub(crate) fn new(addr: sockaddr_un, kind: DBusConnectionKind) -> Self {
        Self {
            fd: -1,
            state: State::CanSocket,
            addr,
            buf: [0; _],
            kind,
        }
    }

    pub(crate) fn dummy(kind: DBusConnectionKind) -> Self {
        Self {
            fd: -1,
            state: State::Dead,
            addr: unsafe { core::mem::MaybeUninit::zeroed().assume_init() },
            buf: [0; _],
            kind,
        }
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        match self.state {
            State::CanSocket => {
                self.state = State::WaitingForSocket;
                Some(Wants::Socket {
                    domain: AF_UNIX,
                    r#type: SOCK_STREAM,
                })
            }
            State::WaitingForSocket => None,

            State::CanConnect => {
                self.state = State::WaitingForConnect;
                Some(Wants::Connect {
                    fd: self.fd,
                    addr: (&self.addr as *const sockaddr_un).cast::<sockaddr>(),
                    addrlen: core::mem::size_of::<sockaddr_un>() as u32,
                })
            }
            State::WaitingForConnect => None,

            State::CanWriteZero => {
                self.state = State::WaitingForWriteZero;
                let buf = b"\0";
                Some(Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                })
            }
            State::WaitingForWriteZero => None,

            State::CanWriteAuthExternal => {
                self.state = State::WaitingForWriteAuthExternal;
                let buf = b"AUTH EXTERNAL\r\n";
                Some(Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                })
            }
            State::WaitingForWriteAuthExternal => None,

            State::CanReadData => {
                self.state = State::WaitingForReadData;
                Some(Wants::Read {
                    fd: self.fd,
                    buf: self.buf.as_mut_ptr(),
                    len: self.buf.len(),
                })
            }
            State::WaitingForReadData => None,

            State::CanWriteData => {
                self.state = State::WaitingForWriteData;
                let buf = b"DATA\r\n";
                Some(Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                })
            }
            State::WaitingForWriteData => None,

            State::CanReadGUID => {
                self.state = State::WaitingForReadGUID;
                Some(Wants::Read {
                    fd: self.fd,
                    buf: self.buf.as_mut_ptr(),
                    len: self.buf.len(),
                })
            }
            State::WaitingForReadGUID => None,

            State::CanWriteBegin => {
                self.state = State::WaitingForWriteBegin;
                let buf = b"BEGIN\r\n";
                Some(Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                })
            }
            State::WaitingForWriteBegin => None,

            State::Done => None,
            State::Dead => None,
        }
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<i32>> {
        match (self.state, satisfy) {
            (State::Dead, _) => Ok(None),

            (State::WaitingForSocket, Satisfy::Socket) => {
                ensure!(res >= 0, "DBusConnector::Socket failed: {res}");
                self.fd = res;
                self.state = State::CanConnect;
                Ok(None)
            }

            (State::WaitingForConnect, Satisfy::Connect) => {
                ensure!(res >= 0, "DBusConnector::Connect failed: {res}");
                self.state = State::CanWriteZero;
                Ok(None)
            }

            (State::WaitingForWriteZero, Satisfy::Write) => {
                ensure!(res >= 0, "DBusConnector::Write failed: {res}");
                let bytes_written = res as usize;
                ensure!(bytes_written == b"\0".len());
                self.state = State::CanWriteAuthExternal;
                Ok(None)
            }

            (State::WaitingForWriteAuthExternal, Satisfy::Write) => {
                ensure!(res >= 0, "DBusConnector::WriteAuthExternal failed: {res}");
                let bytes_written = res as usize;
                ensure!(bytes_written == b"AUTH EXTERNAL\r\n".len());
                self.state = State::CanReadData;
                Ok(None)
            }

            (State::WaitingForReadData, Satisfy::Read) => {
                ensure!(res >= 0, "DBusConnector::ReadData failed: {res}");
                let bytes_read = res as usize;
                ensure!(&self.buf[..bytes_read] == b"DATA\r\n");
                self.state = State::CanWriteData;
                Ok(None)
            }

            (State::WaitingForWriteData, Satisfy::Write) => {
                ensure!(res >= 0, "DBusConnector::WriteData failed: {res}");
                let bytes_written = res as usize;
                ensure!(bytes_written == b"DATA\r\n".len());
                self.state = State::CanReadGUID;
                Ok(None)
            }

            (State::WaitingForReadGUID, Satisfy::Read) => {
                ensure!(res > 0, "DBusConnector::ReadGUID failed: {res}");
                self.state = State::CanWriteBegin;
                Ok(None)
            }

            (State::WaitingForWriteBegin, Satisfy::Write) => {
                ensure!(res >= 0, "DBusConnector::WriteBegin failed: {res}");
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

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Option<i32> {
        match self.try_satisfy(satisfy, res) {
            Ok(fd) => fd,
            Err(err) => {
                log::error!("Module DBusConnector has crashed, stopping: {err:?}");
                self.stop();
                None
            }
        }
    }

    pub(crate) fn stop(&mut self) {
        log::error!("Stopping DBusConnector({:?})", self.kind);
        self.state = State::Dead;
    }
}
