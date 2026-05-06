mod handshake;
mod read_write;
mod request;
mod response;
mod state;

pub(crate) use request::HttpRequest;
pub(crate) use response::HttpResponse;

use crate::sansio::{Dns, Satisfy, Wants};
use anyhow::{Result, bail, ensure};
use handshake::OpenSslHandshake;
use libc::sockaddr_in;
use read_write::OpenSslReadWrite;
use state::OpenSslState;

enum State {
    Dns(Box<Dns>),

    ReadyTo(Action),
    WaitingFor(Action),

    Handshaking(OpenSslHandshake),
    ReadWrite(OpenSslReadWrite),

    Done,
}

#[derive(Debug)]
enum Action {
    Socket,
    Connect,
    Close,
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dns(_) => write!(f, "Dns"),
            Self::ReadyTo(action) => write!(f, "ReadyTo({action:?}"),
            Self::WaitingFor(action) => write!(f, "WaitingFor({action:?}"),
            Self::Handshaking(_) => write!(f, "Handshaking"),
            Self::ReadWrite(_) => write!(f, "ReadWrite"),
            Self::Done => write!(f, "Done"),
        }
    }
}

pub(crate) struct Https {
    state: State,
    addr: sockaddr_in,
    fd: i32,
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
            state: State::Dns(Box::new(Dns::new(domain.as_bytes()))),
            addr: unsafe { std::mem::MaybeUninit::zeroed().assume_init() },
            fd: -1,
            request: request.into_bytes(),
            domain,
            response: vec![],
        }
    }

    pub(crate) const fn is_waiting(&self) -> bool {
        match &self.state {
            State::WaitingFor(_) => true,
            State::Handshaking(handshake) => handshake.is_waiting(),
            State::ReadWrite(read_write) => read_write.is_waiting(),
            State::Dns(_) | State::ReadyTo(_) | State::Done => false,
        }
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        match &mut self.state {
            State::Dns(dns) => dns.wants(),

            State::ReadyTo(Action::Socket) => {
                self.state = State::WaitingFor(Action::Socket);
                Some(Wants::Socket {
                    domain: libc::AF_INET,
                    r#type: libc::SOCK_STREAM,
                })
            }

            State::ReadyTo(Action::Connect) => {
                self.state = State::WaitingFor(Action::Connect);
                Some(Wants::Connect {
                    fd: self.fd,
                    addr: (&raw const self.addr).cast(),
                    addrlen: size_of::<sockaddr_in>() as u32,
                })
            }

            State::Handshaking(handshake) => handshake.wants(),

            State::ReadWrite(ready) => ready.wants(),

            State::ReadyTo(Action::Close) => {
                self.state = State::WaitingFor(Action::Close);
                Some(Wants::Close { fd: self.fd })
            }

            State::WaitingFor(_) | State::Done => None,
        }
    }

    pub(crate) fn try_satisfy(
        &mut self,
        satisfy: Satisfy,
        res: i32,
    ) -> Result<Option<HttpResponse>> {
        match (&mut self.state, satisfy) {
            (State::Dns(dns), _) => {
                if let Some(mut addr) = dns.try_satisfy(satisfy, res)? {
                    addr.sin_port = 443_u16.to_be();
                    self.addr = addr;
                    self.state = State::ReadyTo(Action::Socket);
                }
            }

            (State::WaitingFor(Action::Socket), Satisfy::Socket) => {
                ensure!(res >= 0, "OpenSsl::Socket failed: {res}");
                self.fd = res;
                self.state = State::ReadyTo(Action::Connect);
            }

            (State::WaitingFor(Action::Connect), Satisfy::Connect) => {
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
                    self.response = response;
                    self.state = State::ReadyTo(Action::Close);
                }
            }

            (State::WaitingFor(Action::Close), Satisfy::Close) => {
                ensure!(res >= 0, "close failed: {res}");
                let response = HttpResponse::parse(std::mem::take(&mut self.response))?;
                self.state = State::Done;
                return Ok(Some(response));
            }

            _ => bail!("malformed state: {:?} vs {satisfy:?}", self.state),
        }

        Ok(None)
    }
}
