use crate::{
    sansio::{Satisfy, Wants},
    utils::ArrayWriter,
};
use anyhow::{Context, Result, bail, ensure};
use core::fmt::Write;
use libc::{AF_UNIX, SOCK_STREAM, sockaddr, sockaddr_un};

pub(crate) struct UnixSocketOneshotWriter {
    addr: sockaddr_un,
    buf: [u8; 4_096],
    write_buflen: usize,
    bytes_read: usize,
    fd: i32,
    state: State,
}

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

impl UnixSocketOneshotWriter {
    pub(crate) fn new(addr: sockaddr_un, data: &str) -> Result<Self> {
        let mut buf = [0; 4_096];
        let mut writer = ArrayWriter::new(&mut buf);
        write!(&mut writer, "{data}").context("failed to write command to buffer")?;
        let write_buflen = writer.offset;

        Ok(Self {
            addr,
            buf,
            write_buflen,
            bytes_read: 0,
            fd: -1,
            state: State::ReadyTo(Action::Socket),
        })
    }

    pub(crate) fn wants(&mut self) -> Result<Option<Wants>> {
        let State::ReadyTo(action) = self.state else {
            return Ok(None);
        };

        let wants = match action {
            Action::Socket => Wants::Socket {
                domain: AF_UNIX,
                r#type: SOCK_STREAM,
            },

            Action::Connect => Wants::Connect {
                fd: self.fd,
                addr: (&raw const self.addr).cast::<sockaddr>(),
                addrlen: size_of::<sockaddr_un>() as u32,
            },

            Action::Write => {
                let buf = self
                    .buf
                    .get(..self.write_buflen)
                    .context("buffer is too short")?;
                Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                }
            }

            Action::Read => Wants::Read {
                fd: self.fd,
                buf: self.buf.as_mut_ptr(),
                len: self.buf.len(),
            },

            Action::Close => Wants::Close { fd: self.fd },
        };
        self.state = State::WaitingFor(action);
        Ok(Some(wants))
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<&[u8]>> {
        match (self.state, satisfy) {
            (State::WaitingFor(Action::Socket), Satisfy::Socket) => {
                ensure!(res >= 0, "socket failed: {res}");
                self.fd = res;
                self.state = State::ReadyTo(Action::Connect);
                Ok(None)
            }

            (State::WaitingFor(Action::Connect), Satisfy::Connect) => {
                ensure!(res >= 0, "connect failed: {res}");
                self.state = State::ReadyTo(Action::Write);
                Ok(None)
            }

            (State::WaitingFor(Action::Write), Satisfy::Write) => {
                ensure!(res > 0, "write failed: {res}");
                self.state = State::ReadyTo(Action::Read);
                Ok(None)
            }

            (State::WaitingFor(Action::Read), Satisfy::Read) => {
                self.bytes_read = usize::try_from(res).context("read failed")?;
                self.state = State::ReadyTo(Action::Close);
                Ok(None)
            }

            (State::WaitingFor(Action::Close), Satisfy::Close) => {
                ensure!(res >= 0, "close failed: {res}");
                self.state = State::Done;
                Ok(Some(
                    self.buf
                        .get(..self.bytes_read)
                        .context("buf is too short")?,
                ))
            }

            (state, satisfy) => {
                bail!("malformed state: {state:?} vs {satisfy:?}")
            }
        }
    }

    pub(crate) const fn fd(&self) -> i32 {
        self.fd
    }
}
