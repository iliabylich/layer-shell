mod name;
mod request;
mod response;

use crate::sansio::{Satisfy, Wants};
use anyhow::{Context, Result, bail, ensure};
use request::Request;
use response::Response;

const DNS_SERVER: u32 = 0x08_08_08_08;
const DNS_PORT: u16 = 53;
const TYPE_A: u16 = 1;
const CLASS_IN: u16 = 1;
const MAX_DNS_PACKET: usize = 512;

#[derive(Debug, Clone, Copy)]
enum State {
    ReadyTo(Action),
    WaitingFor(Action),
}

#[derive(Debug, Clone, Copy)]
enum Action {
    Socket,
    Connect,
    Write,
    Read,
    Close,
}

pub(crate) struct Dns {
    state: State,
    fd: i32,
    addr: libc::sockaddr_in,
    buf: [u8; MAX_DNS_PACKET],
    len: usize,
    pos: usize,
    domain: &'static [u8],
}

impl Dns {
    pub(crate) const fn new(domain: &'static [u8]) -> Self {
        Self {
            state: State::ReadyTo(Action::Socket),
            fd: -1,
            addr: libc::sockaddr_in {
                sin_family: libc::AF_INET as u16,
                sin_port: DNS_PORT.to_be(),
                sin_addr: libc::in_addr {
                    s_addr: DNS_SERVER.to_be(),
                },
                sin_zero: [0; 8],
            },
            buf: [0; _],
            len: 0,
            pos: 0,
            domain,
        }
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        match self.state {
            State::ReadyTo(Action::Socket) => {
                self.state = State::WaitingFor(Action::Socket);
                Some(Wants::Socket {
                    domain: libc::AF_INET,
                    r#type: libc::SOCK_DGRAM,
                    seq: 42,
                })
            }

            State::ReadyTo(Action::Connect) => {
                self.state = State::WaitingFor(Action::Connect);
                Some(Wants::Connect {
                    fd: self.fd,
                    addr: (&raw const self.addr).cast::<libc::sockaddr>(),
                    addrlen: size_of::<libc::sockaddr_in>() as u32,
                    seq: 42,
                })
            }

            State::ReadyTo(Action::Write) => {
                self.state = State::WaitingFor(Action::Write);
                // SAFETY: len never exceeds buf's size
                let buf = unsafe { self.buf.get_unchecked(self.pos..self.len) };
                Some(Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                    seq: 42,
                })
            }

            State::ReadyTo(Action::Read) => {
                self.state = State::WaitingFor(Action::Read);
                // SAFETY: len never exceeds buf's size
                let buf = unsafe { self.buf.get_unchecked_mut(self.len..) };
                Some(Wants::Read {
                    fd: self.fd,
                    buf: buf.as_mut_ptr(),
                    len: buf.len(),
                    seq: 42,
                })
            }

            State::ReadyTo(Action::Close) => {
                self.state = State::WaitingFor(Action::Close);
                Some(Wants::Close {
                    fd: self.fd,
                    seq: 42,
                })
            }

            State::WaitingFor(_) => None,
        }
    }

    pub(crate) fn try_satisfy(
        &mut self,
        satisfy: Satisfy,
        res: i32,
    ) -> Result<Option<libc::sockaddr_in>> {
        match (self.state, satisfy) {
            (State::WaitingFor(Action::Socket), Satisfy::Socket) => {
                ensure!(res >= 0);
                self.fd = res;
                self.state = State::ReadyTo(Action::Connect);
                Ok(None)
            }

            (State::WaitingFor(Action::Connect), Satisfy::Connect) => {
                ensure!(res >= 0);

                let mut buf = [0_u8; MAX_DNS_PACKET];
                let len = Request::write(&mut buf, self.domain, TYPE_A);

                self.state = State::ReadyTo(Action::Write);
                self.buf = buf;
                self.len = len;
                self.pos = 0;
                Ok(None)
            }

            (State::WaitingFor(Action::Write), Satisfy::Write) => {
                let bytes_written = usize::try_from(res).context("write failed")?;

                self.pos += bytes_written;
                ensure!(self.pos <= self.len);
                if self.pos == self.len {
                    self.state = State::ReadyTo(Action::Read);
                    self.buf = [0; _];
                    self.len = 0;
                }

                Ok(None)
            }

            (State::WaitingFor(Action::Read), Satisfy::Read) => {
                let bytes_read = usize::try_from(res).context("read failed")?;

                self.len += bytes_read;
                ensure!(self.len < MAX_DNS_PACKET);

                self.state = State::ReadyTo(Action::Close);
                Ok(None)
            }

            (State::WaitingFor(Action::Close), Satisfy::Close) => {
                ensure!(res >= 0);

                let reply = Response::read(self.buf.get(..self.len).context("buf is too short")?)?;
                Ok(Some(reply))
            }

            _ => {
                bail!("malformed state {:?}", self.state)
            }
        }
    }
}
