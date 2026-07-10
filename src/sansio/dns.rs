use crate::{
    external::{__socket_type_SOCK_DGRAM as SOCK_DGRAM, AF_INET, sockaddr_in, socklen_t},
    sansio::{Satisfy, Wants},
    utils::new_sockaddr_in,
};
use alloc::boxed::Box;
use anyhow::{Context, Result, bail};
use core::{
    ffi::CStr,
    mem::size_of,
    net::{Ipv4Addr, SocketAddr},
};
use dns::{Dns, DnsRecordType, DnsWants, MAX_DNS_PACKET_LEN};

#[expect(clippy::upper_case_acronyms)]
pub(crate) struct DNS {
    state: State,
    domain: &'static CStr,
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
    pub(crate) const fn address() -> sockaddr_in {
        new_sockaddr_in(Ipv4Addr::new(8, 8, 8, 8).octets(), 53)
    }

    pub(crate) fn new(domain: &'static CStr) -> Self {
        Self {
            state: State::CanSocket,
            domain,
            output: None,
            buf: Box::new([0; _]),
        }
    }

    pub(crate) fn try_wants(&mut self, addr: &sockaddr_in) -> Result<Option<Wants>> {
        match self.state {
            State::CanSocket => {
                self.state = State::WaitingForSocket;
                Ok(Some(Wants::Socket {
                    domain: AF_INET,
                    type_: SOCK_DGRAM,
                }))
            }

            State::CanConnect { fd } => {
                self.state = State::WaitingForConnect { fd };
                Ok(Some(Wants::Connect {
                    fd,
                    addr: core::ptr::from_ref(addr).cast(),
                    addrlen: size_of::<sockaddr_in>() as socklen_t,
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

    pub(crate) fn try_satisfy(&mut self, satisfy: Satisfy) -> Result<Option<sockaddr_in>> {
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
                    dns: Dns::new(
                        self.domain.to_str().context("non-utf8 domain")?,
                        DnsRecordType::A,
                        &mut self.buf,
                    )?,
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
                let Some(addr) = self.output.take() else {
                    return Ok(None);
                };
                let SocketAddr::V4(addr) = addr else {
                    bail!("DNS query returned non-IPv4 address");
                };
                Ok(Some(new_sockaddr_in(addr.ip().octets(), 443)))
            }

            (_, satisfy) => bail!("malformed state: {:?} vs {satisfy:?}", self.state),
        }
    }
}
