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
    Done,
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

    pub(crate) fn wants(&mut self) -> Result<Option<Wants>> {
        let State::ReadyTo(action) = self.state else {
            return Ok(None);
        };

        let wants = match action {
            Action::Socket => Wants::Socket {
                domain: libc::AF_INET,
                r#type: libc::SOCK_DGRAM,
            },

            Action::Connect => Wants::Connect {
                fd: self.fd,
                addr: (&raw const self.addr).cast::<libc::sockaddr>(),
                addrlen: size_of::<libc::sockaddr_in>() as u32,
            },

            Action::Write => {
                let buf = self
                    .buf
                    .get(self.pos..self.len)
                    .context("buf is too short")?;
                Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                }
            }

            Action::Read => {
                let buf = self.buf.get_mut(self.len..).context("buf is too short")?;
                Wants::Read {
                    fd: self.fd,
                    buf: buf.as_mut_ptr(),
                    len: buf.len(),
                }
            }

            Action::Close => Wants::Close { fd: self.fd },
        };
        self.state = State::WaitingFor(action);
        Ok(Some(wants))
    }

    pub(crate) fn satisfy(
        &mut self,
        satisfy: Satisfy,
        res: i32,
    ) -> Result<Option<libc::sockaddr_in>> {
        let action = match self.state {
            State::WaitingFor(action) => action,
            state => bail!("malformed DNS state: {state:?} vs {satisfy:?}"),
        };

        match (action, satisfy) {
            (Action::Socket, Satisfy::Socket) => {
                ensure!(res >= 0, "DNS::Socket failed: {res}");
                self.fd = res;
                self.state = State::ReadyTo(Action::Connect);
                Ok(None)
            }

            (Action::Connect, Satisfy::Connect) => {
                ensure!(res >= 0, "DNS::Connect failed: {res}");

                let mut buf = [0_u8; MAX_DNS_PACKET];
                let len = Request::write(&mut buf, self.domain, TYPE_A);

                self.state = State::ReadyTo(Action::Write);
                self.buf = buf;
                self.len = len;
                self.pos = 0;
                Ok(None)
            }

            (Action::Write, Satisfy::Write) => {
                ensure!(res >= 0, "DNS::Write failed: {res}");
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

            (Action::Read, Satisfy::Read) => {
                let bytes_read = usize::try_from(res).context("read failed")?;

                self.len += bytes_read;
                ensure!(self.len < MAX_DNS_PACKET);

                self.state = State::ReadyTo(Action::Close);
                Ok(None)
            }

            (Action::Close, Satisfy::Close) => {
                ensure!(res >= 0, "DNS::Close failed: {res}");

                let reply = Response::read(self.buf.get(..self.len).context("buf is too short")?)?;
                self.state = State::Done;
                Ok(Some(reply))
            }

            (state, satisfy) => {
                bail!("malformed DNS state: {state:?} vs {satisfy:?}")
            }
        }
    }
}
