use crate::{
    macros::report_and_exit,
    sansio::{Satisfy, Wants},
};
use anyhow::{Result, bail, ensure};
use libc::{AF_UNIX, SOCK_STREAM, sockaddr, sockaddr_un};
use std::fmt::Write;

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
    CanSocket,
    CanConnect,
    CanWrite,
    CanRead,
    CanClose,
    Done,
}

impl UnixSocketOneshotWriter {
    pub(crate) fn new(addr: sockaddr_un, data: &str) -> Self {
        let mut buf = [0; 4_096];
        let mut writer = ArrayWriter::new(&mut buf);
        write!(&mut writer, "{}", data)
            .unwrap_or_else(|err| report_and_exit!("failed to write command to buffer: {err:?}"));
        let write_buflen = writer.offset;

        Self {
            addr,
            buf,
            write_buflen,
            bytes_read: 0,
            fd: -1,
            state: State::CanSocket,
        }
    }

    pub(crate) fn wants(&mut self) -> Wants<'_> {
        match self.state {
            State::CanSocket => Wants::Socket {
                domain: AF_UNIX,
                r#type: SOCK_STREAM,
            },
            State::CanConnect => Wants::Connect {
                fd: self.fd,
                addr: (&self.addr as *const sockaddr_un).cast::<sockaddr>(),
                addrlen: std::mem::size_of::<sockaddr_un>() as u32,
            },
            State::CanWrite => Wants::Write {
                fd: self.fd,
                buf: &self.buf[..self.write_buflen],
            },
            State::CanRead => Wants::Read {
                fd: self.fd,
                buf: &mut self.buf,
            },
            State::CanClose => Wants::Close { fd: self.fd },
            State::Done => Wants::Nothing,
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<&[u8]>> {
        match (self.state, satisfy) {
            (State::CanSocket, Satisfy::Socket) => {
                ensure!(res >= 0);
                self.fd = res;
                self.state = State::CanConnect;
                Ok(None)
            }

            (State::CanConnect, Satisfy::Connect) => {
                ensure!(res >= 0);
                self.state = State::CanWrite;
                Ok(None)
            }

            (State::CanWrite, Satisfy::Write) => {
                ensure!(res > 0);
                self.state = State::CanRead;
                Ok(None)
            }

            (State::CanRead, Satisfy::Read) => {
                ensure!(res > 0);
                self.bytes_read = res as usize;
                self.state = State::CanClose;
                Ok(None)
            }

            (State::CanClose, Satisfy::Close) => {
                ensure!(res >= 0);
                self.state = State::Done;
                Ok(Some(&self.buf[..self.bytes_read]))
            }

            (state, satisfy) => {
                bail!("malformed UnixSocketOneshotWriter state: {state:?} vs {satisfy:?}")
            }
        }
    }
}

struct ArrayWriter<'a> {
    buf: &'a mut [u8],
    offset: usize,
}
impl<'a> ArrayWriter<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        ArrayWriter { buf, offset: 0 }
    }
}
impl<'a> std::fmt::Write for ArrayWriter<'a> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let bytes = s.as_bytes();

        let remainder = &mut self.buf[self.offset..];
        if remainder.len() < bytes.len() {
            return Err(std::fmt::Error);
        }
        let remainder = &mut remainder[..bytes.len()];
        remainder.copy_from_slice(bytes);

        self.offset += bytes.len();
        Ok(())
    }
}
