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
    CanSocket,
    WaitingForSocket,
    CanConnect,
    WaitingForConnect,
    CanWrite,
    WaitingForWrite,
    CanRead,
    WaitingForRead,
    CanClose,
    WaitingForClose,
    Done,
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
            state: State::CanSocket,
        }
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        match self.state {
            State::CanSocket => {
                self.state = State::WaitingForSocket;
                Some(Wants::Socket {
                    domain: AF_UNIX,
                    r#type: SOCK_STREAM,
                })
            }
            State::WaitingForSocket => None,

            State::CanConnect => {
                self.state = State::WaitingForConnect;
                Some(Wants::Connect {
                    fd: self.fd,
                    addr: (&self.addr as *const sockaddr_un).cast::<sockaddr>(),
                    addrlen: core::mem::size_of::<sockaddr_un>() as u32,
                })
            }
            State::WaitingForConnect => None,

            State::CanWrite => {
                self.state = State::WaitingForWrite;
                let buf = &self.buf[..self.write_buflen];
                Some(Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                })
            }
            State::WaitingForWrite => None,

            State::CanRead => {
                self.state = State::WaitingForRead;
                Some(Wants::Read {
                    fd: self.fd,
                    buf: self.buf.as_mut_ptr(),
                    len: self.buf.len(),
                })
            }
            State::WaitingForRead => None,

            State::CanClose => {
                self.state = State::WaitingForClose;
                Some(Wants::Close { fd: self.fd })
            }
            State::WaitingForClose => None,

            State::Done => None,
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<&[u8]>> {
        match (self.state, satisfy) {
            (State::WaitingForSocket, Satisfy::Socket) => {
                ensure!(res >= 0, "UnixSocketOneshotWriter::Socket failed: {res}");
                self.fd = res;
                self.state = State::CanConnect;
                Ok(None)
            }

            (State::WaitingForConnect, Satisfy::Connect) => {
                ensure!(res >= 0, "UnixSocketOneshotWriter::Connect failed: {res}");
                self.state = State::CanWrite;
                Ok(None)
            }

            (State::WaitingForWrite, Satisfy::Write) => {
                ensure!(res > 0, "UnixSocketOneshotWriter::Write failed: {res}");
                self.state = State::CanRead;
                Ok(None)
            }

            (State::WaitingForRead, Satisfy::Read) => {
                ensure!(res > 0, "UnixSocketOneshotWriter::Read failed: {res}");
                self.bytes_read = res as usize;
                self.state = State::CanClose;
                Ok(None)
            }

            (State::WaitingForClose, Satisfy::Close) => {
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
