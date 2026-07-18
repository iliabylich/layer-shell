use crate::{sansio::Wants, utils::ArrayWriter};
use anyhow::{Context, Result, bail, ensure};
use core::{fmt::Write, mem::size_of};
use libc::{AF_UNIX, SOCK_STREAM, sockaddr_un};

pub(crate) struct UnixSocketOneshotWriter {
    writebuf: [u8; 4_096],
    writebuflen: usize,
    state: State,
}

#[derive(Debug)]
enum State {
    ReadyToSocket,
    WaitingForSocket,

    ReadyToConnect { fd: i32 },
    WaitingForConnect { fd: i32 },

    ReadyToWrite { fd: i32 },
    WaitingForWrite { fd: i32 },

    Done,
}

impl State {
    const fn wants(self, addr: &sockaddr_un, writebuf: &[u8]) -> (Self, Option<Wants>) {
        match self {
            Self::ReadyToSocket => (
                Self::WaitingForSocket,
                Some(Wants::Socket {
                    domain: AF_UNIX,
                    type_: SOCK_STREAM,
                }),
            ),

            Self::ReadyToConnect { fd } => (
                Self::WaitingForConnect { fd },
                Some(Wants::Connect {
                    fd,
                    addr: core::ptr::from_ref(addr).cast(),
                    addrlen: size_of::<sockaddr_un>() as u32,
                }),
            ),

            Self::ReadyToWrite { fd } => (
                Self::WaitingForWrite { fd },
                Some(Wants::Write {
                    fd,
                    buf: writebuf.as_ptr(),
                    len: writebuf.len(),
                }),
            ),

            waiting => (waiting, None),
        }
    }

    const fn wants_in_place(&mut self, addr: &sockaddr_un, writebuf: &[u8]) -> Option<Wants> {
        let mut this = Self::Done;
        core::mem::swap(self, &mut this);
        let (next, wants) = this.wants(addr, writebuf);
        *self = next;
        wants
    }
}

impl UnixSocketOneshotWriter {
    pub(crate) fn new(data: &str) -> Result<Self> {
        let mut writebuf = [0; 4_096];
        let mut writer = ArrayWriter::new(&mut writebuf);
        write!(&mut writer, "{data}").context("failed to write command to buffer")?;
        let writebuflen = writer.offset;

        Ok(Self {
            writebuf,
            writebuflen,
            state: State::ReadyToSocket,
        })
    }

    pub(crate) fn wants(&mut self, addr: &sockaddr_un) -> Option<Wants> {
        self.state.wants_in_place(
            addr,
            self.writebuf
                .get(..self.writebuflen)
                .unwrap_or_else(|| unreachable!()),
        )
    }

    pub(crate) fn satisfy_socket(&mut self, fd: i32) -> Result<()> {
        ensure!(
            matches!(self.state, State::WaitingForSocket),
            "malformed state: expected Socket, got {:?}",
            self.state
        );

        self.state = State::ReadyToConnect { fd };
        Ok(())
    }

    pub(crate) fn satisfy_connect(&mut self) -> Result<()> {
        let State::WaitingForConnect { fd } = self.state else {
            bail!("malformed state: expected Connect, got {:?}", self.state)
        };

        self.state = State::ReadyToWrite { fd };
        Ok(())
    }

    pub(crate) fn satisfy_write(&self) -> Result<()> {
        let State::WaitingForWrite { .. } = self.state else {
            bail!("malformed state: expected Write, got {:?}", self.state)
        };

        Ok(())
    }

    pub(crate) fn fd(&self) -> Result<i32> {
        match self.state {
            State::ReadyToConnect { fd }
            | State::WaitingForConnect { fd }
            | State::ReadyToWrite { fd }
            | State::WaitingForWrite { fd } => Ok(fd),

            State::ReadyToSocket | State::WaitingForSocket | State::Done => bail!(
                "UnixSocketOneshotWriter doesn't have FD in {:?} state",
                self.state
            ),
        }
    }
}
