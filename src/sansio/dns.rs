use crate::sansio::{Satisfy, Wants};
use alloc::boxed::Box;
use anyhow::{Result, bail};
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use dns::{Dns, DnsRecordType, DnsWants, MAX_DNS_PACKET_LEN};
use rustix::net::SocketAddrAny;

#[expect(clippy::upper_case_acronyms)]
pub(crate) struct DNS {
    state: State,
    domain: &'static str,
    output: Option<SocketAddr>,
    buf: Box<[u8; MAX_DNS_PACKET_LEN]>,
}

enum State {
    CanSocket,
    WaitingForSocket,

    CanConnect { fd: i32 },
    WaitingForConnect { fd: i32 },

    CanWrite { dns: Dns, fd: i32 },
    WaitingForWrite { dns: Dns, fd: i32 },

    CanRead { dns: Dns, fd: i32 },
    WaitingForRead { dns: Dns, fd: i32 },

    CanClose { fd: i32 },
    WaitingForClose,

    Finished,
}
impl core::fmt::Debug for State {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::CanSocket => write!(f, "CanSocket"),
            Self::WaitingForSocket => write!(f, "WaitingForSocket"),
            Self::CanConnect { .. } => write!(f, "CanConnect"),
            Self::WaitingForConnect { .. } => write!(f, "WaitingForConnect"),
            Self::CanWrite { .. } => write!(f, "CanWrite"),
            Self::WaitingForWrite { .. } => write!(f, "WaitingForWrite"),
            Self::CanRead { .. } => write!(f, "CanRead"),
            Self::WaitingForRead { .. } => write!(f, "WaitingForRead"),
            Self::CanClose { .. } => write!(f, "CanClose"),
            Self::WaitingForClose => write!(f, "WaitingForClose"),
            Self::Finished => write!(f, "Stopped"),
        }
    }
}

impl DNS {
    pub(crate) fn address() -> SocketAddrAny {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 53).into()
    }

    pub(crate) fn new(domain: &'static str) -> Self {
        Self {
            state: State::CanSocket,
            domain,
            output: None,
            buf: Box::new([0; _]),
        }
    }

    pub(crate) fn try_wants(&mut self, addr: &SocketAddrAny) -> Result<Option<Wants>> {
        match self.state {
            State::CanSocket => {
                self.state = State::WaitingForSocket;
                Ok(Some(Wants::Socket {
                    domain: libc::AF_INET,
                    type_: libc::SOCK_DGRAM,
                }))
            }

            State::CanConnect { fd } => {
                self.state = State::WaitingForConnect { fd };
                Ok(Some(Wants::Connect {
                    fd,
                    addr: addr.as_ptr().cast(),
                    addrlen: addr.addr_len(),
                }))
            }

            State::CanWrite { mut dns, fd } => {
                let Some(DnsWants::Write { buf, .. }) = dns.wants(&mut self.buf)? else {
                    bail!("DNS at state CanWrite must want Write")
                };
                self.state = State::WaitingForWrite { dns, fd };
                Ok(Some(Wants::Write {
                    fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                }))
            }

            State::CanRead { mut dns, fd } => {
                let Some(DnsWants::Read { buf, .. }) = dns.wants(&mut self.buf)? else {
                    bail!("DNS at state CanRead must want Read")
                };
                self.state = State::WaitingForRead { dns, fd };
                Ok(Some(Wants::Read {
                    fd,
                    buf: buf.as_mut_ptr(),
                    len: buf.len(),
                }))
            }

            State::CanClose { fd } => {
                self.state = State::WaitingForClose;
                Ok(Some(Wants::Close { fd }))
            }

            State::WaitingForSocket
            | State::WaitingForConnect { .. }
            | State::WaitingForWrite { .. }
            | State::WaitingForRead { .. }
            | State::WaitingForClose
            | State::Finished => Ok(None),
        }
    }

    pub(crate) fn try_satisfy(&mut self, satisfy: Satisfy) -> Result<Option<SocketAddrAny>> {
        let mut state = State::Finished;
        core::mem::swap(&mut self.state, &mut state);

        match (state, satisfy) {
            (State::WaitingForSocket, Satisfy::Socket(res)) => {
                let fd = res?;
                self.state = State::CanConnect { fd };
                Ok(None)
            }

            (State::WaitingForConnect { fd }, Satisfy::Connect(res)) => {
                res?;
                self.state = State::CanWrite {
                    dns: Dns::new(self.domain, DnsRecordType::A, &mut self.buf)?,
                    fd,
                };
                Ok(None)
            }

            (State::WaitingForWrite { mut dns, fd }, Satisfy::Write(res)) => {
                let len = res?;
                dns.satisfy_write(len, &mut self.buf)?;
                self.state = State::CanRead { dns, fd };
                Ok(None)
            }

            (State::WaitingForRead { mut dns, fd }, Satisfy::Read(res)) => {
                let len = res?;
                let (addr, _seq) = dns.satisfy_read(len, &self.buf)?;
                self.state = State::CanClose { fd };
                self.output = Some(addr);
                Ok(None)
            }

            (State::WaitingForClose, Satisfy::Close(res)) => {
                res?;
                let Some(mut addr) = self.output.take() else {
                    return Ok(None);
                };
                addr.set_port(443);
                Ok(Some(addr.into()))
            }

            (_, satisfy) => bail!("malformed state: {:?} vs {satisfy:?}", self.state),
        }
    }
}
