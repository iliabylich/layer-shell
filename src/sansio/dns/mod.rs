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
    CanConnect {
        fd: i32,
        addr: libc::sockaddr_in,
    },
    CanWrite {
        fd: i32,
        buf: [u8; MAX_DNS_PACKET],
        len: usize,
        pos: usize,
    },
    CanRead {
        fd: i32,
        buf: [u8; MAX_DNS_PACKET],
        len: usize,
    },
    CanClose {
        fd: i32,
        buf: [u8; MAX_DNS_PACKET],
        len: usize,
    },
    Done,
}

pub(crate) struct Dns {
    state: State,
    domain: &'static [u8],
}

impl Dns {
    pub(crate) fn new(domain: &'static [u8]) -> Self {
        Self {
            state: State::CanSocket,
            domain,
        }
    }

    pub(crate) fn wants(&mut self) -> Wants {
        match &mut self.state {
            State::CanSocket => Wants::Socket {
                domain: libc::AF_INET,
                r#type: libc::SOCK_DGRAM,
            },
            State::CanConnect { fd, addr } => Wants::Connect {
                fd: *fd,
                addr: (addr as *const libc::sockaddr_in).cast::<libc::sockaddr>(),
                addrlen: core::mem::size_of::<libc::sockaddr_in>() as u32,
            },
            State::CanWrite { fd, buf, len, pos } => {
                let buf = &buf[*pos..*len];
                Wants::Write {
                    fd: *fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                }
            }
            State::CanRead { fd, buf, len } => {
                let buf = &mut buf[*len..];
                Wants::Read {
                    fd: *fd,
                    buf: buf.as_mut_ptr(),
                    len: buf.len(),
                }
            }
            State::CanClose { fd, .. } => Wants::Close { fd: *fd },
            State::Done => Wants::Nothing,
        }
    }

    pub(crate) fn satisfy(
        &mut self,
        satisfy: Satisfy,
        res: i32,
    ) -> Result<Option<libc::sockaddr_in>> {
        match (&mut self.state, satisfy) {
            (State::CanSocket, Satisfy::Socket) => {
                ensure!(res >= 0, "DNS::Socket failed: {res}");
                let fd = res;

                let addr = libc::sockaddr_in {
                    sin_family: libc::AF_INET as u16,
                    sin_port: DNS_PORT.to_be(),
                    sin_addr: libc::in_addr {
                        s_addr: DNS_SERVER.to_be(),
                    },
                    sin_zero: [0; 8],
                };

                self.state = State::CanConnect { fd, addr };
                Ok(None)
            }

            (State::CanConnect { fd, .. }, Satisfy::Connect) => {
                ensure!(res >= 0, "DNS::Connect failed: {res}");

                let mut buf = [0_u8; MAX_DNS_PACKET];
                let len = Request::write(&mut buf, self.domain, TYPE_A);

                self.state = State::CanWrite {
                    fd: *fd,
                    buf,
                    len,
                    pos: 0,
                };
                Ok(None)
            }

            (State::CanWrite { fd, len, pos, .. }, Satisfy::Write) => {
                ensure!(res >= 0, "DNS::Write failed: {res}");
                let bytes_written = res as usize;

                *pos += bytes_written;
                ensure!(*pos <= *len);
                if *pos == *len {
                    self.state = State::CanRead {
                        fd: *fd,
                        buf: [0; _],
                        len: 0,
                    };
                }

                Ok(None)
            }

            (State::CanRead { fd, buf, len }, Satisfy::Read) => {
                ensure!(res >= 0, "DNS::Read failed: {res}");
                let bytes_read = res as usize;

                *len += bytes_read;
                ensure!(*len < MAX_DNS_PACKET);

                self.state = State::CanClose {
                    fd: *fd,
                    buf: *buf,
                    len: *len,
                };
                Ok(None)
            }

            (State::CanClose { buf, len, .. }, Satisfy::Close) => {
                ensure!(res >= 0, "DNS::Close failed: {res}");

                let reply = Response::read(&buf[..*len])?;
                self.state = State::Done;
                Ok(Some(reply))
            }

            (state, satisfy) => {
                bail!("malformed DNS state: {state:?} vs {satisfy:?}")
            }
        }
    }
}
