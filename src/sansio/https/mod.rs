mod handshake;
mod read_write;
mod request;
mod response;
mod state;

use std::{net::SocketAddr, os::fd::BorrowedFd};

pub(crate) use request::HttpRequest;
pub(crate) use response::HttpResponse;

use anyhow::{Result, bail};
use handshake::OpenSslHandshake;
use read_write::OpenSslReadWrite;
use rustix::net::{AddressFamily, SocketType};
use state::OpenSslState;

use crate::sansio::{DNS, Satisfy, Wants};

enum State {
    Dns(DNS),

    ReadyToSocket {
        addr: SocketAddr,
    },
    WaitingForSocket {
        addr: SocketAddr,
    },

    ReadyToConnect {
        addr: SocketAddr,
        fd: BorrowedFd<'static>,
    },
    WaitingForConnect {
        fd: BorrowedFd<'static>,
    },

    Handshaking {
        inner: OpenSslHandshake,
        fd: BorrowedFd<'static>,
    },
    ReadWrite {
        inner: OpenSslReadWrite,
        fd: BorrowedFd<'static>,
    },

    ReadyToClose {
        fd: BorrowedFd<'static>,
    },
    WaitingForClose,

    Done,
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dns(_) => write!(f, "Dns"),
            Self::ReadyToSocket { .. } => write!(f, "ReadyToSocket"),
            Self::WaitingForSocket { .. } => write!(f, "WaitingForSocket"),
            Self::ReadyToConnect { .. } => write!(f, "ReadyToConnect"),
            Self::WaitingForConnect { .. } => write!(f, "WaitingForConnect"),
            Self::Handshaking { .. } => write!(f, "Handshaking"),
            Self::ReadWrite { .. } => write!(f, "ReadWrite"),
            Self::ReadyToClose { .. } => write!(f, "ReadyToClose"),
            Self::WaitingForClose => write!(f, "WaitingForClose"),
            Self::Done => write!(f, "Done"),
        }
    }
}

pub(crate) struct Https {
    state: State,
    seq: u64,
    request: Vec<u8>,
    domain: &'static str,
    response: Vec<u8>,
}

impl Https {
    pub(crate) fn init() -> Result<()> {
        OpenSslState::init()
    }

    pub(crate) fn new(request: HttpRequest) -> Self {
        let domain = request.host();

        Self {
            state: State::Dns(DNS::new(domain)),
            seq: 0,
            request: request.into_bytes(),
            domain,
            response: vec![],
        }
    }

    pub(crate) const fn is_waiting(&self) -> bool {
        match &self.state {
            State::WaitingForSocket { .. }
            | State::WaitingForConnect { .. }
            | State::WaitingForClose => true,
            State::Handshaking { inner, .. } => inner.is_waiting(),
            State::ReadWrite { inner, .. } => inner.is_waiting(),
            State::Dns(_)
            | State::ReadyToSocket { .. }
            | State::ReadyToConnect { .. }
            | State::ReadyToClose { .. }
            | State::Done => false,
        }
    }

    pub(crate) fn wants(&mut self) -> Result<Option<Wants>> {
        match &mut self.state {
            State::Dns(dns) => dns.wants(),

            State::ReadyToSocket { addr } => {
                self.state = State::WaitingForSocket { addr: *addr };
                Ok(Some(Wants::Socket {
                    domain: AddressFamily::INET,
                    r#type: SocketType::STREAM,
                    seq: self.seq,
                }))
            }

            State::ReadyToConnect { addr, fd } => {
                let addr = *addr;
                let fd = *fd;
                self.state = State::WaitingForConnect { fd };
                Ok(Some(Wants::Connect {
                    fd,
                    addr: addr.into(),
                    seq: self.seq,
                }))
            }

            State::Handshaking { inner, fd } => Ok(inner.wants(*fd)),

            State::ReadWrite { inner, fd } => Ok(inner.wants(*fd)),

            State::ReadyToClose { fd } => {
                let fd = *fd;
                self.state = State::WaitingForClose;
                Ok(Some(Wants::Close { fd, seq: self.seq }))
            }

            State::WaitingForSocket { .. }
            | State::WaitingForConnect { .. }
            | State::WaitingForClose
            | State::Done => Ok(None),
        }
    }

    pub(crate) fn try_satisfy(&mut self, satisfy: Satisfy) -> Result<Option<HttpResponse>> {
        match (&mut self.state, satisfy) {
            (State::Dns(dns), satisfy) => {
                if let Some((seq, mut addr)) = dns.try_satisfy(satisfy)? {
                    self.seq = seq;
                    addr.set_port(443);
                    self.state = State::ReadyToSocket { addr };
                }
            }

            (State::WaitingForSocket { addr }, Satisfy::Socket(res)) => {
                let fd = res?;
                self.seq += 1;
                self.state = State::ReadyToConnect { addr: *addr, fd };
            }

            (State::WaitingForConnect { fd }, Satisfy::Connect(res)) => {
                res?;
                let fd = *fd;
                self.seq += 1;
                self.state = State::Handshaking {
                    inner: OpenSslHandshake::new(self.domain, self.seq)?,
                    fd,
                };
            }

            (State::Handshaking { inner, fd }, satisfy) => {
                if let Some((state, seq)) = inner.satisfy(satisfy)? {
                    let fd = *fd;
                    self.state = State::ReadWrite {
                        inner: OpenSslReadWrite::new(state, self.request.clone(), seq)?,
                        fd,
                    };
                }
            }

            (State::ReadWrite { inner, fd }, satisfy) => {
                if let Some(response) = inner.satisfy(satisfy)? {
                    let fd = *fd;
                    self.response = response;
                    self.state = State::ReadyToClose { fd };
                }
            }

            (State::WaitingForClose, Satisfy::Close(res)) => {
                res?;
                let response = HttpResponse::parse(std::mem::take(&mut self.response))?;
                self.state = State::Done;
                return Ok(Some(response));
            }

            (_, satisfy) => bail!("malformed state: {:?} vs {satisfy:?}", self.state),
        }

        Ok(None)
    }
}
