#[expect(non_camel_case_types, clippy::upper_case_acronyms)]
mod generated;

mod handshake;
mod read_write;
mod request;
mod response;
mod state;

use alloc::{vec, vec::Vec};
use generated::SSL_CTX;
pub(crate) use request::HttpRequest;
pub(crate) use response::HttpResponse;
pub(crate) use state::OpenSslContext;

use anyhow::{Result, bail};
use handshake::OpenSslHandshake;
use read_write::OpenSslReadWrite;
use rustix::net::{AddressFamily, SocketAddrAny, SocketType};
use state::OpenSslState;

use crate::sansio::{Satisfy, Wants};

enum State {
    CanSocket,
    WaitingForSocket,

    CanConnect { fd: i32 },
    WaitingForConnect { fd: i32 },

    Handshaking { inner: OpenSslHandshake, fd: i32 },
    ReadWrite { inner: OpenSslReadWrite, fd: i32 },

    CanClose { fd: i32 },
    WaitingForClose,

    Finished,
}

impl core::fmt::Debug for State {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
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
    pub(crate) fn new(request: HttpRequest, ctx: &OpenSslContext) -> Result<Self> {
        let domain = request.host();

        Ok(Self {
            state: State::CanSocket,
            ctx: ctx.raw(),
            request: request.into_bytes()?,
            domain,
            response: vec![],
        })
    }

    pub(crate) fn wants(&mut self, remote_server_addr: &SocketAddrAny) -> Option<Wants> {
        match &mut self.state {
            State::CanSocket => {
                self.state = State::WaitingForSocket;
                Some(Wants::Socket {
                    domain: AddressFamily::INET,
                    r#type: SocketType::STREAM,
                })
            }

            State::CanConnect { fd } => {
                let fd = *fd;
                self.state = State::WaitingForConnect { fd };
                Some(Wants::Connect {
                    fd,
                    addr: remote_server_addr.clone(),
                })
            }

            State::Handshaking { inner, fd } => inner.wants(*fd),

            State::ReadWrite { inner, fd } => inner.wants(*fd),

            State::CanClose { fd } => {
                let fd = *fd;
                self.state = State::WaitingForClose;
                Some(Wants::Close { fd })
            }

            State::WaitingForSocket
            | State::WaitingForConnect { .. }
            | State::WaitingForClose
            | State::Finished => None,
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy) -> Result<Option<HttpResponse>> {
        match (&mut self.state, satisfy) {
            (State::WaitingForSocket, Satisfy::Socket(res)) => {
                let fd = res?;
                self.state = State::CanConnect { fd };
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
                let response = HttpResponse::parse(core::mem::take(&mut self.response))?;
                self.state = State::Finished;
                return Ok(Some(response));
            }

            (_, satisfy) => bail!("malformed state: {:?} vs {satisfy:?}", self.state),
        }

        Ok(None)
    }
}
