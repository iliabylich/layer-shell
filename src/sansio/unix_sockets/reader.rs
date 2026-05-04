use crate::sansio::{Satisfy, Wants};
use anyhow::{Context, Result, bail, ensure};
use libc::{AF_UNIX, SOCK_STREAM, sockaddr, sockaddr_un};

#[derive(Debug, Clone, Copy)]
enum State {
    ReadyTo(Action),
    WaitingFor(Action),
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
    pub(crate) const fn new(addr: sockaddr_un) -> Self {
        Self {
            addr,
            fd: -1,
            buf: [0; _],
            state: State::ReadyTo(Action::Socket),
        }
    }

    pub(crate) const fn new_connected_from_fd(fd: i32) -> Self {
        let mut this = Self::dummy();
        this.fd = fd;
        this.state = State::ReadyTo(Action::Read);
        this
    }

    pub(crate) const fn dummy() -> Self {
        Self {
            addr: unsafe { std::mem::MaybeUninit::zeroed().assume_init() },
            fd: -1,
            buf: [0; _],
            state: State::ReadyTo(Action::Socket),
        }
    }

    pub(crate) const fn wants(&mut self) -> Option<Wants> {
        match self.state {
            State::ReadyTo(Action::Socket) => {
                self.state = State::WaitingFor(Action::Socket);
                Some(Wants::Socket {
                    domain: AF_UNIX,
                    r#type: SOCK_STREAM,
                })
            }

            State::ReadyTo(Action::Connect) => {
                self.state = State::WaitingFor(Action::Connect);
                Some(Wants::Connect {
                    fd: self.fd,
                    addr: (&raw const self.addr).cast::<sockaddr>(),
                    addrlen: size_of::<sockaddr_un>() as u32,
                })
            }

            State::ReadyTo(Action::Read) => {
                self.state = State::WaitingFor(Action::Read);
                Some(Wants::Read {
                    fd: self.fd,
                    buf: self.buf.as_mut_ptr(),
                    len: self.buf.len(),
                })
            }

            State::WaitingFor(_) => None,
        }
    }

    pub(crate) fn try_satisfy(
        &mut self,
        satisfy: Satisfy,
        res: i32,
    ) -> Result<Option<([u8; 1_024], usize)>> {
        match (self.state, satisfy) {
            (State::WaitingFor(Action::Socket), Satisfy::Socket) => {
                ensure!(res >= 0, "Socket failed: {res}");
                self.fd = res;
                self.state = State::ReadyTo(Action::Connect);
                Ok(None)
            }

            (State::WaitingFor(Action::Connect), Satisfy::Connect) => {
                ensure!(res >= 0, "Connect failed: {res}");
                self.state = State::ReadyTo(Action::Read);
                Ok(None)
            }

            (State::WaitingFor(Action::Read), Satisfy::Read) => {
                let bytes_read = usize::try_from(res).context("Read failed")?;
                ensure!(bytes_read != 0, "EOF");
                let buf = self.buf;
                self.buf = [0; _];
                self.state = State::ReadyTo(Action::Read);
                Ok(Some((buf, bytes_read)))
            }

            (state, satisfy) => {
                bail!("malformed state: {state:?} vs {satisfy:?}");
            }
        }
    }
}
