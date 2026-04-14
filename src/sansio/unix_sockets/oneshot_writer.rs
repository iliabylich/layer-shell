use crate::{
    sansio::{Satisfy, Wants},
    utils::{ArrayWriter, StringRef, report_and_exit},
};
use anyhow::{Result, bail, ensure};
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
    pub(crate) fn new(addr: sockaddr_un, data: StringRef) -> Self {
        let mut buf = [0; 4_096];
        let mut writer = ArrayWriter::new(&mut buf);
        write!(&mut writer, "{}", data).unwrap_or_else(|err: core::fmt::Error| {
            report_and_exit!("failed to write command to buffer: {err:?}")
        });
        let write_buflen = writer.offset;

        Self {
            addr,
            buf,
            write_buflen,
            bytes_read: 0,
            fd: -1,
            state: State::ReadyTo(Action::Socket),
        }
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        let State::ReadyTo(action) = self.state else {
            return None;
        };

        let wants = match action {
            Action::Socket => Wants::Socket {
                domain: AF_UNIX,
                r#type: SOCK_STREAM,
            },

            Action::Connect => Wants::Connect {
                fd: self.fd,
                addr: (&self.addr as *const sockaddr_un).cast::<sockaddr>(),
                addrlen: core::mem::size_of::<sockaddr_un>() as u32,
            },

            Action::Write => {
                let buf = &self.buf[..self.write_buflen];
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
        Some(wants)
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<&[u8]>> {
        let action = match self.state {
            State::WaitingFor(action) => action,
            state => bail!("malformed UnixSocketOneshotWriter state: {state:?} vs {satisfy:?}"),
        };

        match (action, satisfy) {
            (Action::Socket, Satisfy::Socket) => {
                ensure!(res >= 0, "UnixSocketOneshotWriter::Socket failed: {res}");
                self.fd = res;
                self.state = State::ReadyTo(Action::Connect);
                Ok(None)
            }

            (Action::Connect, Satisfy::Connect) => {
                ensure!(res >= 0, "UnixSocketOneshotWriter::Connect failed: {res}");
                self.state = State::ReadyTo(Action::Write);
                Ok(None)
            }

            (Action::Write, Satisfy::Write) => {
                ensure!(res > 0, "UnixSocketOneshotWriter::Write failed: {res}");
                self.state = State::ReadyTo(Action::Read);
                Ok(None)
            }

            (Action::Read, Satisfy::Read) => {
                ensure!(res > 0, "UnixSocketOneshotWriter::Read failed: {res}");
                self.bytes_read = res as usize;
                self.state = State::ReadyTo(Action::Close);
                Ok(None)
            }

            (Action::Close, Satisfy::Close) => {
                ensure!(res >= 0, "UnixSocketOneshotWriter::Close failed: {res}");
                self.state = State::Done;
                Ok(Some(&self.buf[..self.bytes_read]))
            }

            (state, satisfy) => {
                bail!("malformed UnixSocketOneshotWriter state: {state:?} vs {satisfy:?}")
            }
        }
    }
}
