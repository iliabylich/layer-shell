mod name;
mod request;
mod response;

use crate::sansio::Wants;
use anyhow::{Context, Result, ensure};
use request::Request;
use response::Response;

const DNS_SERVER: u32 = 0x08_08_08_08;
const DNS_PORT: u16 = 53;
const TYPE_A: u16 = 1;
const CLASS_IN: u16 = 1;
const MAX_DNS_PACKET: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Socket,
    Connect,
    Write,
    Read,
    Close,
}

pub(crate) struct Dns {
    state: State,
    seq: u64,
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
            state: State::Socket,
            seq: 0,
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

    pub(crate) fn wants(&mut self) -> Result<Wants> {
        match self.state {
            State::Socket => Ok(Wants::Socket {
                domain: libc::AF_INET,
                r#type: libc::SOCK_DGRAM,
                seq: self.seq,
            }),

            State::Connect => Ok(Wants::Connect {
                fd: self.fd,
                addr: (&raw const self.addr).cast::<libc::sockaddr>(),
                addrlen: size_of::<libc::sockaddr_in>() as u32,
                seq: self.seq,
            }),

            State::Write => {
                let buf = self.buf.get(self.pos..self.len).context("internal error")?;
                Ok(Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                    seq: self.seq,
                })
            }

            State::Read => {
                let buf = self.buf.get_mut(self.len..).context("internal error")?;
                Ok(Wants::Read {
                    fd: self.fd,
                    buf: buf.as_mut_ptr(),
                    len: buf.len(),
                    seq: self.seq,
                })
            }

            State::Close => Ok(Wants::Close {
                fd: self.fd,
                seq: self.seq,
            }),
        }
    }

    pub(crate) fn satisfy_socket(&mut self, res: i32) -> Result<()> {
        ensure!(
            self.state == State::Socket,
            "malformed state, expected Socket, got {:?}",
            self.state
        );

        ensure!(res >= 0);
        self.fd = res;
        self.state = State::Connect;
        self.seq += 1;
        Ok(())
    }

    pub(crate) fn satisfy_connect(&mut self, res: i32) -> Result<()> {
        ensure!(
            self.state == State::Connect,
            "malformed state, expected Connect, got {:?}",
            self.state
        );

        ensure!(res >= 0);

        let mut buf = [0_u8; MAX_DNS_PACKET];
        let len = Request::write(&mut buf, self.domain, TYPE_A);

        self.state = State::Write;
        self.seq += 1;

        self.buf = buf;
        self.len = len;
        self.pos = 0;
        Ok(())
    }

    pub(crate) fn satisfy_write(&mut self, res: i32) -> Result<()> {
        ensure!(
            self.state == State::Write,
            "malformed state, expected Write, got {:?}",
            self.state
        );

        let bytes_written = usize::try_from(res).context("write failed")?;

        self.pos += bytes_written;
        self.seq += 1;
        ensure!(self.pos <= self.len);
        if self.pos == self.len {
            self.state = State::Read;
            self.buf = [0; _];
            self.len = 0;
        }
        Ok(())
    }

    pub(crate) fn satisfy_read(&mut self, res: i32) -> Result<()> {
        ensure!(
            self.state == State::Read,
            "malformed state, expected Read, got {:?}",
            self.state
        );

        let bytes_read = usize::try_from(res).context("read failed")?;

        self.len += bytes_read;
        self.seq += 1;
        ensure!(self.len < MAX_DNS_PACKET);

        self.state = State::Close;
        Ok(())
    }

    pub(crate) fn satisfy_close(&mut self, res: i32) -> Result<(libc::sockaddr_in, u64)> {
        ensure!(
            self.state == State::Close,
            "malformed state, expected Close, got {:?}",
            self.state
        );

        ensure!(res >= 0);
        self.seq += 1;

        let mut addr = Response::read(self.buf.get(..self.len).context("buf is too short")?)?;
        addr.sin_port = 443_u16.to_be();
        Ok((addr, self.seq))
    }
}
