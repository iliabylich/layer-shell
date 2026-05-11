use crate::sansio::Wants;
use anyhow::{Context, Result, ensure};
use libc::{AF_UNIX, SOCK_STREAM, sockaddr, sockaddr_un};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Socket,
    Connect,
    Read,
}

pub(crate) struct UnixSocketReader {
    addr: sockaddr_un,
    fd: i32,
    buf: [u8; 1_024],
    state: State,
    seq: u64,
}

impl UnixSocketReader {
    pub(crate) const fn new(addr: sockaddr_un) -> Self {
        Self {
            addr,
            fd: -1,
            buf: [0; _],
            state: State::Socket,
            seq: 0,
        }
    }

    pub(crate) const fn new_connected_from_fd(fd: i32) -> Self {
        Self {
            addr: unsafe { std::mem::MaybeUninit::zeroed().assume_init() },
            fd,
            buf: [0; _],
            state: State::Read,
            seq: 0,
        }
    }

    pub(crate) const fn dummy() -> Self {
        Self {
            addr: unsafe { std::mem::MaybeUninit::zeroed().assume_init() },
            fd: -1,
            buf: [0; _],
            state: State::Socket,
            seq: 0,
        }
    }

    pub(crate) const fn wants(&mut self) -> Wants {
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

            State::Read => Wants::Read {
                fd: self.fd,
                buf: self.buf.as_mut_ptr(),
                len: self.buf.len(),
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

        ensure!(res >= 0, "Socket failed: {res}");
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

        ensure!(res >= 0, "Connect failed: {res}");
        self.state = State::Read;
        self.seq += 1;
        Ok(())
    }

    pub(crate) fn satisfy_read(&mut self, res: i32) -> Result<([u8; 1_024], usize)> {
        ensure!(
            self.state == State::Read,
            "malformed state: expected Read, got {:?}",
            self.state
        );

        let bytes_read = usize::try_from(res).context("Read failed")?;
        ensure!(bytes_read != 0, "EOF");
        let buf = self.buf;
        self.buf = [0; _];
        self.seq += 1;

        Ok((buf, bytes_read))
    }
}
