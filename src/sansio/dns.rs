use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use anyhow::{Context as _, Result, bail, ensure};
use dns::{Dns, DnsRecordType, DnsWants};
use libc::{sockaddr, sockaddr_in};
use rustix::net::{AddressFamily, SocketType};

use crate::sansio::{Satisfy, Wants};

#[expect(clippy::upper_case_acronyms)]
pub(crate) struct DNS {
    state: State,
    domain: &'static str,
    dns_server_fd: Option<i32>,
    dns_server_sock_addr: sockaddr_in,
    seq: u64,
    output: Option<SocketAddr>,
}

const DNS_SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 53);

enum State {
    Socket,
    Connect,
    ReadWrite(Box<Dns>),
    Close,
}
impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Socket => write!(f, "Socket"),
            Self::Connect => write!(f, "Connect"),
            Self::ReadWrite(_) => write!(f, "ReadWrite"),
            Self::Close => write!(f, "Close"),
        }
    }
}

impl DNS {
    pub(crate) fn new(domain: &'static str) -> Self {
        Self {
            state: State::Socket,
            domain,
            dns_server_fd: None,
            dns_server_sock_addr: socketaddr_to_sockaddr_in4(DNS_SERVER_ADDR)
                .unwrap_or_else(|| unreachable!()),
            seq: 0,
            output: None,
        }
    }

    pub(crate) fn wants(&mut self) -> Result<Option<Wants>> {
        match &mut self.state {
            State::Socket => Ok(Some(Wants::Socket {
                domain: AddressFamily::INET.as_raw().into(),
                r#type: i32::try_from(SocketType::DGRAM.as_raw())
                    .context("malformed socket type")?,
                seq: self.seq,
            })),

            State::Connect => {
                let addr = (&raw const self.dns_server_sock_addr).cast::<sockaddr>();

                let fd = self
                    .dns_server_fd
                    .unwrap_or_else(|| unreachable!("FD must be set at this point, bug"));

                Ok(Some(Wants::Connect {
                    fd,
                    addr,
                    addrlen: size_of::<sockaddr_in>() as u32,
                    seq: self.seq,
                }))
            }

            State::ReadWrite(inner) => {
                match inner.wants()?.context("DNS doesn't want anything")? {
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
                }
            }

            State::Close => {
                let fd = self
                    .dns_server_fd
                    .unwrap_or_else(|| unreachable!("FD must be set at this point, bug"));
                Ok(Some(Wants::Close { fd, seq: self.seq }))
            }
        }
    }

    pub(crate) fn try_satisfy(
        &mut self,
        satisfy: Satisfy,
        res: i32,
    ) -> Result<Option<(u64, SocketAddr)>> {
        self.seq += 1;

        match (&mut self.state, satisfy) {
            (State::Socket, Satisfy::Socket) => {
                ensure!(res >= 0, "DNS socket failed");
                self.dns_server_fd = Some(res);
                self.state = State::Connect;
                Ok(None)
            }

            (State::Connect, Satisfy::Connect) => {
                ensure!(res >= 0, "DNS connect failed");
                self.state = State::ReadWrite(Box::new(Dns::new(self.domain, DnsRecordType::A)?));
                Ok(None)
            }

            (State::ReadWrite(inner), Satisfy::Write) => {
                let len = usize::try_from(res).context("DNS write failed")?;
                inner.satisfy_write(len)?;
                Ok(None)
            }

            (State::ReadWrite(inner), Satisfy::Read) => {
                let len = usize::try_from(res).context("DNS read failed")?;
                let (addr, _seq) = inner.satisfy_read(len)?;
                self.state = State::Close;
                self.output = Some(addr);
                Ok(None)
            }

            (State::Close, Satisfy::Close) => {
                ensure!(res >= 0);
                let Some(output) = self.output.take() else {
                    return Ok(None);
                };
                Ok(Some((self.seq, output)))
            }

            _ => bail!("malformed state: {:?} vs {satisfy:?}", self.state),
        }
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
