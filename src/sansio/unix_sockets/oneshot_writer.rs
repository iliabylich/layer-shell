use crate::{sansio::Wants, utils::ArrayWriter};
use anyhow::{Context, Result, ensure};
use core::fmt::Write;
use libc::{AF_UNIX, SOCK_STREAM, sockaddr, sockaddr_un};

pub(crate) struct UnixSocketOneshotWriter {
    addr: sockaddr_un,
    buf: [u8; 4_096],
    write_buflen: usize,
    bytes_read: usize,
    fd: i32,
    state: State,
    seq: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
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
            state: State::Socket,
            seq: 0,
        })
    }

    pub(crate) fn wants(&mut self) -> Wants {
        match self.state {
            State::Socket => Wants::Socket {
                domain: AF_UNIX,
                r#type: SOCK_STREAM,
                seq: self.seq,
            },

            State::Connect => Wants::Connect {
                fd: self.fd,
                addr: (&raw const self.addr).cast::<sockaddr>(),
                addrlen: size_of::<sockaddr_un>() as u32,
                seq: self.seq,
            },

            State::Write => {
                // SAFETY: write_buflen is guaranteed not to exceed buf's len
                let buf = unsafe { self.buf.get_unchecked(..self.write_buflen) };
                Wants::Write {
                    fd: self.fd,
                    buf: buf.as_ptr(),
                    len: buf.len(),
                    seq: self.seq,
                }
            }

            State::Read => Wants::Read {
                fd: self.fd,
                buf: self.buf.as_mut_ptr(),
                len: self.buf.len(),
                seq: self.seq,
            },

            State::Close => Wants::Close {
                fd: self.fd,
                seq: self.seq,
            },
        }
    }

    pub(crate) fn satisfy_socket(&mut self, res: i32) -> Result<()> {
        ensure!(
            self.state == State::Socket,
            "malformed state: expected Socket, got {:?}",
            self.state
        );

        ensure!(res >= 0, "socket failed: {res}");
        self.fd = res;
        self.state = State::Connect;
        self.seq += 1;
        Ok(())
    }

    pub(crate) fn satisfy_connect(&mut self, res: i32) -> Result<()> {
        ensure!(
            self.state == State::Connect,
            "malformed state: expected Connect, got {:?}",
            self.state
        );

        ensure!(res >= 0, "connect failed: {res}");
        self.state = State::Write;
        self.seq += 1;
        Ok(())
    }

    pub(crate) fn satisfy_write(&mut self, res: i32) -> Result<()> {
        ensure!(
            self.state == State::Write,
            "malformed state: expected Write, got {:?}",
            self.state
        );

        ensure!(res > 0, "write failed: {res}");
        self.state = State::Read;
        self.seq += 1;
        Ok(())
    }

    #[expect(dead_code)]
    pub(crate) fn satisfy_read(&mut self, res: i32) -> Result<()> {
        ensure!(
            self.state == State::Write,
            "malformed state: expected Write, got {:?}",
            self.state
        );

        self.bytes_read = usize::try_from(res).context("read failed")?;
        self.state = State::Close;
        self.seq += 1;
        Ok(())
    }

    #[expect(dead_code)]
    pub(crate) fn satisfy_close(&mut self, res: i32) -> Result<&[u8]> {
        ensure!(
            self.state == State::Close,
            "malformed state: expected Close, got {:?}",
            self.state
        );

        ensure!(res >= 0, "close failed: {res}");
        self.seq += 1;
        self.buf.get(..self.bytes_read).context("buf is too short")
    }

    pub(crate) const fn fd(&self) -> i32 {
        self.fd
    }
}
