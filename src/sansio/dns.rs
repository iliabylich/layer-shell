use crate::sansio::{Satisfy, Wants};
use anyhow::{Context as _, Result, bail};
use dns::{Dns, DnsRecordType, DnsWants};
use rustix::net::{AddressFamily, SocketType};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    os::fd::BorrowedFd,
};

#[expect(clippy::upper_case_acronyms)]
pub(crate) struct DNS {
    state: State,
    domain: &'static str,
    dns_server_fd: Option<BorrowedFd<'static>>,
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
    pub(crate) const fn new(domain: &'static str) -> Self {
        Self {
            state: State::Socket,
            domain,
            dns_server_fd: None,
            seq: 0,
            output: None,
        }
    }

    pub(crate) fn wants(&mut self) -> Result<Option<Wants>> {
        match &mut self.state {
            State::Socket => Ok(Some(Wants::Socket {
                domain: AddressFamily::INET,
                r#type: SocketType::DGRAM,
                seq: self.seq,
            })),

            State::Connect => {
                let fd = self
                    .dns_server_fd
                    .unwrap_or_else(|| unreachable!("FD must be set at this point, bug"));

                Ok(Some(Wants::Connect {
                    fd,
                    addr: DNS_SERVER_ADDR.into(),
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

    pub(crate) fn try_satisfy(&mut self, satisfy: Satisfy) -> Result<Option<(u64, SocketAddr)>> {
        self.seq += 1;

        match (&mut self.state, satisfy) {
            (State::Socket, Satisfy::Socket(res)) => {
                let fd = res?;
                self.dns_server_fd = Some(fd);
                self.state = State::Connect;
                Ok(None)
            }

            (State::Connect, Satisfy::Connect(res)) => {
                res?;
                self.state = State::ReadWrite(Box::new(Dns::new(self.domain, DnsRecordType::A)?));
                Ok(None)
            }

            (State::ReadWrite(inner), Satisfy::Write(res)) => {
                let len = res?;
                inner.satisfy_write(len)?;
                Ok(None)
            }

            (State::ReadWrite(inner), Satisfy::Read(res)) => {
                let len = res?;
                let (addr, _seq) = inner.satisfy_read(len)?;
                self.state = State::Close;
                self.output = Some(addr);
                Ok(None)
            }

            (State::Close, Satisfy::Close(res)) => {
                res?;
                let Some(output) = self.output.take() else {
                    return Ok(None);
                };
                Ok(Some((self.seq, output)))
            }

            (_, satisfy) => bail!("malformed state: {:?} vs {satisfy:?}", self.state),
        }
    }
}
