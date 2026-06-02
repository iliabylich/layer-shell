mod handshake;
mod read_write;
mod request;
mod response;
mod state;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use dns::{Dns, DnsRecordType, DnsWants};
pub(crate) use request::HttpRequest;
pub(crate) use response::HttpResponse;

use anyhow::{Context as _, Result, bail, ensure};
use handshake::OpenSslHandshake;
use libc::{sockaddr, sockaddr_in};
use read_write::OpenSslReadWrite;
use state::OpenSslState;

use crate::sansio::{Satisfy, Wants};

enum State {
    Dns(Box<Dns<'static>>),

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
    seq: u64,
    addr: sockaddr_in,
    fd: i32,
    request: Vec<u8>,
    domain: &'static str,
    response: Vec<u8>,

    dns_server_fd: Option<i32>,
    dns_server_sock_addr: Option<sockaddr_in>,
}

impl Https {
    pub(crate) fn init() -> Result<()> {
        OpenSslState::init()
    }

    pub(crate) fn new(request: HttpRequest) -> Self {
        let domain = request.host();

        Self {
            state: State::Dns(Box::new(Dns::new(
                domain,
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 53),
                DnsRecordType::A,
            ))),
            seq: 0,
            addr: unsafe { std::mem::MaybeUninit::zeroed().assume_init() },
            fd: -1,
            request: request.into_bytes(),
            domain,
            response: vec![],

            dns_server_fd: None,
            dns_server_sock_addr: None,
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

    pub(crate) fn wants(&mut self) -> Result<Option<Wants>> {
        match &mut self.state {
            State::Dns(dns) => match dns.wants() {
                DnsWants::Socket {
                    domain,
                    r#type,
                    seq,
                } => Ok(Some(Wants::Socket {
                    domain: domain.as_raw().into(),
                    r#type: i32::try_from(r#type.as_raw()).context("malformed socket type")?,
                    seq,
                })),
                DnsWants::Connect { addr, seq } => {
                    self.dns_server_sock_addr = Some(
                        socketaddr_to_sockaddr_in4(addr).context("received non ipv6 IP address")?,
                    );
                    let addr = self
                        .dns_server_sock_addr
                        .as_ref()
                        .map(|addr| (&raw const *addr).cast::<sockaddr>())
                        .unwrap_or_else(|| unreachable!("it is set 1 line above"));
                    let fd = self
                        .dns_server_fd
                        .unwrap_or_else(|| unreachable!("FD must be set at this point, bug"));

                    Ok(Some(Wants::Connect {
                        fd,
                        addr,
                        addrlen: size_of::<sockaddr_in>() as u32,
                        seq,
                    }))
                }
                DnsWants::Read { buf, seq } => {
                    let fd = self
                        .dns_server_fd
                        .unwrap_or_else(|| unreachable!("FD must be set at this point, bug"));
                    Ok(Some(Wants::Read {
                        fd,
                        buf: buf.as_mut_ptr(),
                        len: buf.len(),
                        seq,
                    }))
                }
                DnsWants::Write { buf, seq } => {
                    let fd = self
                        .dns_server_fd
                        .unwrap_or_else(|| unreachable!("FD must be set at this point, bug"));
                    Ok(Some(Wants::Write {
                        fd,
                        buf: buf.as_ptr(),
                        len: buf.len(),
                        seq,
                    }))
                }
                DnsWants::Close { seq } => {
                    let fd = self
                        .dns_server_fd
                        .unwrap_or_else(|| unreachable!("FD must be set at this point, bug"));
                    Ok(Some(Wants::Close { fd, seq }))
                }
            },

            State::ReadyTo(Action::Socket) => {
                self.state = State::WaitingFor(Action::Socket);
                Ok(Some(Wants::Socket {
                    domain: libc::AF_INET,
                    r#type: libc::SOCK_STREAM,
                    seq: self.seq,
                }))
            }

            State::ReadyTo(Action::Connect) => {
                self.state = State::WaitingFor(Action::Connect);
                Ok(Some(Wants::Connect {
                    fd: self.fd,
                    addr: (&raw const self.addr).cast(),
                    addrlen: size_of::<sockaddr_in>() as u32,
                    seq: self.seq,
                }))
            }

            State::Handshaking(handshake) => Ok(handshake.wants()),

            State::ReadWrite(ready) => Ok(ready.wants()),

            State::ReadyTo(Action::Close) => {
                self.state = State::WaitingFor(Action::Close);
                Ok(Some(Wants::Close {
                    fd: self.fd,
                    seq: self.seq,
                }))
            }

            State::WaitingFor(_) | State::Done => Ok(None),
        }
    }

    pub(crate) fn try_satisfy(
        &mut self,
        satisfy: Satisfy,
        res: i32,
    ) -> Result<Option<HttpResponse>> {
        match (&mut self.state, satisfy) {
            (State::Dns(dns), _) => match satisfy {
                Satisfy::Socket => {
                    ensure!(res >= 0, "DNS socket failed");
                    self.dns_server_fd = Some(res);
                    dns.satisfy_socket()?;
                }
                Satisfy::Connect => {
                    ensure!(res >= 0, "DNS connect failed");
                    dns.satisfy_connect()?;
                }
                Satisfy::Write => {
                    let len = usize::try_from(res).context("DNS write failed")?;
                    dns.satisfy_write(len)?;
                }
                Satisfy::Read => {
                    let len = usize::try_from(res).context("DNS read failed")?;
                    dns.satisfy_read(len)?;
                }
                Satisfy::Close => {
                    ensure!(res >= 0);
                    let (addr, seq) = dns.satisfy_close()?;
                    self.addr = socketaddr_to_sockaddr_in4(addr)
                        .context("DNS name wasn't resolved to IPv4")?;
                    self.addr.sin_port = 443_u16.to_be();
                    self.seq = seq;
                    self.state = State::ReadyTo(Action::Socket);
                }
                Satisfy::OpenAt => bail!("DNS module doesn't support Satisfy::{satisfy:?}"),
            },

            (State::WaitingFor(Action::Socket), Satisfy::Socket) => {
                ensure!(res >= 0, "OpenSsl::Socket failed: {res}");
                self.fd = res;
                self.seq += 1;
                self.state = State::ReadyTo(Action::Connect);
            }

            (State::WaitingFor(Action::Connect), Satisfy::Connect) => {
                ensure!(res >= 0, "OpenSsl::Connect failed: {res}");

                self.seq += 1;
                self.state =
                    State::Handshaking(OpenSslHandshake::new(self.fd, self.domain, self.seq)?);
            }

            (State::Handshaking(handshake), Satisfy::Read | Satisfy::Write) => {
                if let Some((state, seq)) = handshake.satisfy(satisfy, res)? {
                    self.state = State::ReadWrite(OpenSslReadWrite::new(
                        self.fd,
                        state,
                        self.request.clone(),
                        seq,
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

const fn socketaddr_to_sockaddr_in4(addr: SocketAddr) -> Option<sockaddr_in> {
    let SocketAddr::V4(addr) = addr else {
        return None;
    };

    let mut sin: sockaddr_in = unsafe { std::mem::zeroed() };
    sin.sin_family = libc::AF_INET as libc::sa_family_t;
    sin.sin_port = addr.port().to_be();
    sin.sin_addr = libc::in_addr {
        s_addr: u32::from_ne_bytes(addr.ip().octets()),
    };
    Some(sin)
}
