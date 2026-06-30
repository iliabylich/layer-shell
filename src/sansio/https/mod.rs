mod handshake;
mod read_write;
mod request;
mod response;
mod state;

use std::{net::SocketAddr, os::fd::BorrowedFd};

use openssl_sys::SSL_CTX;
pub(crate) use request::HttpRequest;
pub(crate) use response::HttpResponse;
pub(crate) use state::OpenSslContext;

use anyhow::{Result, bail};
use handshake::OpenSslHandshake;
use read_write::OpenSslReadWrite;
use rustix::net::{AddressFamily, SocketType};
use state::OpenSslState;

use crate::sansio::{DNS, Satisfy, Wants};

enum State {
    Dns(DNS),

    CanSocket {
        addr: SocketAddr,
    },
    WaitingForSocket {
        addr: SocketAddr,
    },

    CanConnect {
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

    CanClose {
        fd: BorrowedFd<'static>,
    },
    WaitingForClose,

    Finished,
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dns(_) => write!(f, "Dns"),
            Self::CanSocket { .. } => write!(f, "CanSocket"),
            Self::WaitingForSocket { .. } => write!(f, "WaitingForSocket"),
            Self::CanConnect { .. } => write!(f, "CanConnect"),
            Self::WaitingForConnect { .. } => write!(f, "WaitingForConnect"),
            Self::Handshaking { .. } => write!(f, "Handshaking"),
            Self::ReadWrite { .. } => write!(f, "ReadWrite"),
            Self::CanClose { .. } => write!(f, "CanClose"),
            Self::WaitingForClose => write!(f, "WaitingForClose"),
            Self::Finished => write!(f, "Stopped"),
        }
    }
}

pub(crate) struct Https {
    state: State,
    ctx: *mut SSL_CTX,
    request: Vec<u8>,
    domain: &'static str,
    response: Vec<u8>,
}

impl Https {
    pub(crate) fn new(request: HttpRequest, ctx: &OpenSslContext) -> Self {
        let domain = request.host();

        Self {
            state: State::Dns(DNS::new(domain)),
            ctx: ctx.raw(),
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
            | State::CanSocket { .. }
            | State::CanConnect { .. }
            | State::CanClose { .. }
            | State::Finished => false,
        }
    }

    pub(crate) fn try_wants(&mut self) -> Result<Option<Wants>> {
        match &mut self.state {
            State::Dns(dns) => dns.try_wants(),

            State::CanSocket { addr } => {
                self.state = State::WaitingForSocket { addr: *addr };
                Ok(Some(Wants::Socket {
                    domain: AddressFamily::INET,
                    r#type: SocketType::STREAM,
                }))
            }

            State::CanConnect { addr, fd } => {
                let addr = *addr;
                let fd = *fd;
                self.state = State::WaitingForConnect { fd };
                Ok(Some(Wants::Connect {
                    fd,
                    addr: addr.into(),
                }))
            }

            State::Handshaking { inner, fd } => Ok(inner.wants(*fd)),

            State::ReadWrite { inner, fd } => Ok(inner.wants(*fd)),

            State::CanClose { fd } => {
                let fd = *fd;
                self.state = State::WaitingForClose;
                Ok(Some(Wants::Close { fd }))
            }

            State::WaitingForSocket { .. }
            | State::WaitingForConnect { .. }
            | State::WaitingForClose
            | State::Finished => Ok(None),
        }
    }

    pub(crate) fn try_satisfy(&mut self, satisfy: Satisfy) -> Result<Option<HttpResponse>> {
        match (&mut self.state, satisfy) {
            (State::Dns(dns), satisfy) => {
                if let Some(mut addr) = dns.try_satisfy(satisfy)? {
                    addr.set_port(443);
                    self.state = State::CanSocket { addr };
                }
            }

            (State::WaitingForSocket { addr }, Satisfy::Socket(res)) => {
                let fd = res?;
                self.state = State::CanConnect { addr: *addr, fd };
            }

            (State::WaitingForConnect { fd }, Satisfy::Connect(res)) => {
                res?;
                let fd = *fd;
                self.state = State::Handshaking {
                    inner: OpenSslHandshake::new(self.domain, self.ctx)?,
                    fd,
                };
            }

            (State::Handshaking { inner, fd }, satisfy) => {
                if let Some(state) = inner.satisfy(satisfy)? {
                    let fd = *fd;
                    self.state = State::ReadWrite {
                        inner: OpenSslReadWrite::new(state, self.request.clone())?,
                        fd,
                    };
                }
            }

            (State::ReadWrite { inner, fd }, satisfy) => {
                if let Some(response) = inner.satisfy(satisfy)? {
                    let fd = *fd;
                    self.response = response;
                    self.state = State::CanClose { fd };
                }
            }

            (State::WaitingForClose, Satisfy::Close(res)) => {
                res?;
                let response = HttpResponse::parse(std::mem::take(&mut self.response))?;
                self.state = State::Finished;
                return Ok(Some(response));
            }

            (_, satisfy) => bail!("malformed state: {:?} vs {satisfy:?}", self.state),
        }

        Ok(None)
    }
}
