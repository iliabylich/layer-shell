use crate::sansio::{Satisfy, Wants};
use anyhow::{Result, bail, ensure};
use libc::{AF_UNIX, SOCK_STREAM, sockaddr, sockaddr_un};

#[derive(Debug, Clone, Copy)]
enum State {
    ReadyTo(Action),
    WaitingFor(Action),

    Dead,
}

#[derive(Debug, Clone, Copy)]
enum Action {
    Socket,
    Connect,
    Read,
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
            state: State::ReadyTo(Action::Socket),
        }
    }

    pub(crate) fn dummy() -> Self {
        Self {
            addr: unsafe { std::mem::MaybeUninit::zeroed().assume_init() },
            fd: -1,
            buf: [0; _],
            state: State::ReadyTo(Action::Socket),
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

            Action::Read => Wants::Read {
                fd: self.fd,
                buf: self.buf.as_mut_ptr(),
                len: self.buf.len(),
            },
        };
        self.state = State::WaitingFor(action);
        Some(wants)
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<([u8; 1_024], usize)>> {
        let waiting_for = match self.state {
            State::WaitingFor(waiting_for) => waiting_for,
            State::Dead => return Ok(None),
            state => bail!("malformed UnixSocketReader state: {state:?} vs {satisfy:?}"),
        };

        match (waiting_for, satisfy) {
            (Action::Socket, Satisfy::Socket) => {
                ensure!(res >= 0, "UnixSocketReader::Socket failed: {res}");
                self.fd = res;
                self.state = State::ReadyTo(Action::Connect);
                Ok(None)
            }

            (Action::Connect, Satisfy::Connect) => {
                ensure!(res >= 0, "UnixSocketReader::Connect failed: {res}");
                self.state = State::ReadyTo(Action::Read);
                Ok(None)
            }

            (Action::Read, Satisfy::Read) => {
                ensure!(res > 0, "UnixSocketReader::Read failed: {res}");
                let len = res as usize;
                let buf = self.buf;
                self.buf = [0; _];
                self.state = State::ReadyTo(Action::Read);
                Ok(Some((buf, len)))
            }

            (action, satisfy) => {
                bail!("malformed UnixSocketReader state: waiting for {action:?} vs {satisfy:?}");
            }
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Option<([u8; 1_024], usize)> {
        match self.try_satisfy(satisfy, res) {
            Ok(buf) => buf,
            Err(err) => {
                log::error!("Module UnixSocketReader has crashed: {satisfy:?} {res} {err:?}");
                self.stop();
                None
            }
        }
    }

    pub(crate) fn stop(&mut self) {
        log::error!("Stopping UnixSocketReader");
        self.state = State::Dead;
    }
}
