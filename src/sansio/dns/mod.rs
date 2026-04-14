mod name;
mod request;
mod response;

use crate::sansio::{Satisfy, Wants};
use anyhow::{Result, bail, ensure};
use request::Request;
use response::Response;

const DNS_SERVER: u32 = 0x08_08_08_08;
const DNS_PORT: u16 = 53;
const TYPE_A: u16 = 1;
const CLASS_IN: u16 = 1;
const MAX_DNS_PACKET: usize = 512;

#[derive(Debug)]
enum State {
    CanSocket,
    WaitingForSocket,
    CanConnect,
    WaitingForConnect,
    CanWrite,
    WaitingForWrite,
    CanRead,
    WaitingFoRead,
    CanClose,
    WaitingForClose,
    Done,
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
    pub(crate) fn new(domain: &'static [u8]) -> Self {
        Self {
            state: State::CanSocket,
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
            State::CanSocket => {
                self.state = State::WaitingForSocket;

                Some(Wants::Socket {
                    domain: libc::AF_INET,
                    r#type: libc::SOCK_DGRAM,
                })
            }
            State::WaitingForSocket => None,

            State::CanConnect => {
                self.state = State::WaitingForConnect;
                Some(Wants::Connect {
                    fd: self.fd,
                    addr: (&self.addr as *const libc::sockaddr_in).cast::<libc::sockaddr>(),
                    addrlen: core::mem::size_of::<libc::sockaddr_in>() as u32,
                })
            }
            State::WaitingForConnect => None,

            State::CanWrite => {
                self.state = State::WaitingForWrite;
                let buf = &self.buf[self.pos..self.len];
                Some(Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                })
            }
            State::WaitingForWrite => None,

            State::CanRead => {
                self.state = State::WaitingFoRead;
                let buf = &mut self.buf[self.len..];
                Some(Wants::Read {
                    fd: self.fd,
                    buf: buf.as_mut_ptr(),
                    len: buf.len(),
                })
            }
            State::WaitingFoRead => None,

            State::CanClose => {
                self.state = State::WaitingForClose;
                Some(Wants::Close { fd: self.fd })
            }
            State::WaitingForClose => None,

            State::Done => None,
        }
    }

    pub(crate) fn satisfy(
        &mut self,
        satisfy: Satisfy,
        res: i32,
    ) -> Result<Option<libc::sockaddr_in>> {
        match (&mut self.state, satisfy) {
            (State::WaitingForSocket, Satisfy::Socket) => {
                ensure!(res >= 0, "DNS::Socket failed: {res}");
                self.fd = res;
                self.state = State::CanConnect;
                Ok(None)
            }

            (State::WaitingForConnect, Satisfy::Connect) => {
                ensure!(res >= 0, "DNS::Connect failed: {res}");

                let mut buf = [0_u8; MAX_DNS_PACKET];
                let len = Request::write(&mut buf, self.domain, TYPE_A);

                self.state = State::CanWrite;
                self.buf = buf;
                self.len = len;
                self.pos = 0;
                Ok(None)
            }

            (State::WaitingForWrite, Satisfy::Write) => {
                ensure!(res >= 0, "DNS::Write failed: {res}");
                let bytes_written = res as usize;

                self.pos += bytes_written;
                ensure!(self.pos <= self.len);
                if self.pos == self.len {
                    self.state = State::CanRead;
                    self.buf = [0; _];
                    self.len = 0;
                }

                Ok(None)
            }

            (State::WaitingFoRead, Satisfy::Read) => {
                ensure!(res >= 0, "DNS::Read failed: {res}");
                let bytes_read = res as usize;

                self.len += bytes_read;
                ensure!(self.len < MAX_DNS_PACKET);

                self.state = State::CanClose;
                Ok(None)
            }

            (State::WaitingForClose, Satisfy::Close) => {
                ensure!(res >= 0, "DNS::Close failed: {res}");

                let reply = Response::read(&self.buf[..self.len])?;
                self.state = State::Done;
                Ok(Some(reply))
            }

            (state, satisfy) => {
                bail!("malformed DNS state: {state:?} vs {satisfy:?}")
            }
        }
    }
}
