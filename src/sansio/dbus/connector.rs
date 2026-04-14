use crate::sansio::{DBusConnectionKind, Satisfy, Wants};
use anyhow::{Result, bail, ensure};
use libc::{AF_UNIX, SOCK_STREAM, sockaddr, sockaddr_un};

#[derive(Debug, Clone, Copy)]
enum State {
    ReadyTo(Action),
    WaitingFor(Action),
    Done,
    Dead,
}

#[derive(Debug, Clone, Copy)]
enum Action {
    Socket,
    Connect,
    WriteZero,
    WriteAuthExternal,
    ReadData,
    WriteData,
    ReadGUID,
    WriteBegin,
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
            state: State::ReadyTo(Action::Socket),
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
        let State::ReadyTo(action) = self.state else {
            return None;
        };

        let wants = match action {
            Action::Socket => Wants::Socket {
                domain: AF_UNIX,
                r#type: SOCK_STREAM,
            },

            Action::Connect => Wants::Connect {
                fd: self.fd,
                addr: (&self.addr as *const sockaddr_un).cast::<sockaddr>(),
                addrlen: core::mem::size_of::<sockaddr_un>() as u32,
            },

            Action::WriteZero => {
                let buf = b"\0";
                Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                }
            }

            Action::WriteAuthExternal => {
                let buf = b"AUTH EXTERNAL\r\n";
                Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                }
            }

            Action::ReadData => Wants::Read {
                fd: self.fd,
                buf: self.buf.as_mut_ptr(),
                len: self.buf.len(),
            },

            Action::WriteData => {
                let buf = b"DATA\r\n";
                Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                }
            }

            Action::ReadGUID => Wants::Read {
                fd: self.fd,
                buf: self.buf.as_mut_ptr(),
                len: self.buf.len(),
            },

            Action::WriteBegin => {
                let buf = b"BEGIN\r\n";
                Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                }
            }
        };
        self.state = State::WaitingFor(action);
        Some(wants)
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<i32>> {
        let action = match self.state {
            State::WaitingFor(action) => action,
            State::Dead => return Ok(None),
            state => bail!("malformed DBusConnector state: {state:?} vs {satisfy:?}"),
        };

        match (action, satisfy) {
            (Action::Socket, Satisfy::Socket) => {
                ensure!(res >= 0, "DBusConnector::Socket failed: {res}");
                self.fd = res;
                self.state = State::ReadyTo(Action::Connect);
                Ok(None)
            }

            (Action::Connect, Satisfy::Connect) => {
                ensure!(res >= 0, "DBusConnector::Connect failed: {res}");
                self.state = State::ReadyTo(Action::WriteZero);
                Ok(None)
            }

            (Action::WriteZero, Satisfy::Write) => {
                ensure!(res >= 0, "DBusConnector::Write failed: {res}");
                let bytes_written = res as usize;
                ensure!(bytes_written == b"\0".len());
                self.state = State::ReadyTo(Action::WriteAuthExternal);
                Ok(None)
            }

            (Action::WriteAuthExternal, Satisfy::Write) => {
                ensure!(res >= 0, "DBusConnector::WriteAuthExternal failed: {res}");
                let bytes_written = res as usize;
                ensure!(bytes_written == b"AUTH EXTERNAL\r\n".len());
                self.state = State::ReadyTo(Action::ReadData);
                Ok(None)
            }

            (Action::ReadData, Satisfy::Read) => {
                ensure!(res >= 0, "DBusConnector::ReadData failed: {res}");
                let bytes_read = res as usize;
                ensure!(&self.buf[..bytes_read] == b"DATA\r\n");
                self.state = State::ReadyTo(Action::WriteData);
                Ok(None)
            }

            (Action::WriteData, Satisfy::Write) => {
                ensure!(res >= 0, "DBusConnector::WriteData failed: {res}");
                let bytes_written = res as usize;
                ensure!(bytes_written == b"DATA\r\n".len());
                self.state = State::ReadyTo(Action::ReadGUID);
                Ok(None)
            }

            (Action::ReadGUID, Satisfy::Read) => {
                ensure!(res > 0, "DBusConnector::ReadGUID failed: {res}");
                self.state = State::ReadyTo(Action::WriteBegin);
                Ok(None)
            }

            (Action::WriteBegin, Satisfy::Write) => {
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
