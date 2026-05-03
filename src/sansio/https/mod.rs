mod handshake;
mod read_write;
mod request;
mod response;
mod state;

pub(crate) use request::HttpRequest;
pub(crate) use response::HttpResponse;

use crate::sansio::{Dns, Satisfy, Wants};
use anyhow::{Result, ensure};
use handshake::OpenSslHandshake;
use libc::sockaddr_in;
use read_write::OpenSslReadWrite;
use state::OpenSslState;

enum State {
    Dns(Box<Dns>),

    ReadyToSocket,
    WaitingForSocket,
    ReadyToConnect,
    WaitingForConnect,

    Handshaking(OpenSslHandshake),
    ReadWrite(OpenSslReadWrite),

    Done,
}

pub(crate) struct Https {
    state: State,
    addr: sockaddr_in,
    fd: i32,
    request: Vec<u8>,
    domain: &'static str,
}

impl Https {
    pub(crate) fn init() -> Result<()> {
        OpenSslState::init()
    }

    pub(crate) fn new(request: HttpRequest) -> Self {
        let domain = request.host();

        Self {
            state: State::Dns(Box::new(Dns::new(domain.as_bytes()))),
            addr: unsafe { std::mem::MaybeUninit::zeroed().assume_init() },
            fd: -1,
            request: request.into_bytes(),
            domain,
        }
    }

    pub(crate) fn wants(&mut self) -> Result<Option<Wants>> {
        match &mut self.state {
            State::Dns(dns) => dns.wants(),

            State::ReadyToSocket => {
                self.state = State::WaitingForSocket;
                Ok(Some(Wants::Socket {
                    domain: libc::AF_INET,
                    r#type: libc::SOCK_STREAM,
                }))
            }

            State::ReadyToConnect => {
                self.state = State::WaitingForConnect;
                Ok(Some(Wants::Connect {
                    fd: self.fd,
                    addr: (&raw const self.addr).cast(),
                    addrlen: size_of::<sockaddr_in>() as u32,
                }))
            }

            State::Handshaking(handshake) => Ok(handshake.wants()),

            State::ReadWrite(ready) => Ok(ready.wants()),

            State::WaitingForConnect | State::WaitingForSocket | State::Done => Ok(None),
        }
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<HttpResponse>> {
        match (&mut self.state, satisfy) {
            (State::Dns(dns), _) => {
                if let Some(mut addr) = dns.satisfy(satisfy, res)? {
                    addr.sin_port = 443_u16.to_be();
                    self.addr = addr;
                    self.state = State::ReadyToSocket;
                }
            }

            (State::WaitingForSocket, Satisfy::Socket) => {
                ensure!(res >= 0, "OpenSsl::Socket failed: {res}");
                self.fd = res;
                self.state = State::ReadyToConnect;
            }

            (State::WaitingForConnect, Satisfy::Connect) => {
                ensure!(res >= 0, "OpenSsl::Connect failed: {res}");

                self.state = State::Handshaking(OpenSslHandshake::new(self.fd, self.domain)?);
            }

            (State::Handshaking(handshake), Satisfy::Read | Satisfy::Write) => {
                if let Some(state) = handshake.satisfy(satisfy, res)? {
                    self.state = State::ReadWrite(OpenSslReadWrite::new(
                        self.fd,
                        state,
                        self.request.clone(),
                    )?);
                }
            }

            (State::ReadWrite(ready), Satisfy::Read | Satisfy::Write) => {
                if let Some(response) = ready.satisfy(satisfy, res)? {
                    self.state = State::Done;
                    return Ok(Some(HttpResponse::parse(response)?));
                }
            }

            _ => unreachable!(),
        }

        Ok(None)
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Option<HttpResponse> {
        match self.try_satisfy(satisfy, res) {
            Ok(res) => res,
            Err(err) => {
                log::error!("{err:?}");
                self.state = State::Done;
                None
            }
        }
    }
}
