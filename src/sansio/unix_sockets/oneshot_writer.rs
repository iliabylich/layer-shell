use crate::{sansio::Wants, utils::ArrayWriter};
use anyhow::{Context, Result, bail, ensure};
use core::fmt::Write;
use rustix::net::{AddressFamily, SocketAddrUnix, SocketType};
use std::os::fd::BorrowedFd;

pub(crate) struct UnixSocketOneshotWriter {
    addr: SocketAddrUnix,
    writebuf: [u8; 4_096],
    writebuflen: usize,
    readbuf: [u8; 4_096],
    readbuflen: usize,
    state: State,
    seq: u64,
}

#[derive(Debug)]
enum State {
    ReadyToSocket,
    WaitingForSocket,

    ReadyToConnect { fd: BorrowedFd<'static> },
    WaitingForConnect { fd: BorrowedFd<'static> },

    ReadyToWrite { fd: BorrowedFd<'static> },
    WaitingForWrite { fd: BorrowedFd<'static> },

    ReadyToRead { fd: BorrowedFd<'static> },
    WaitingForRead { fd: BorrowedFd<'static> },

    ReadyToClose { fd: BorrowedFd<'static> },
    WaitingForClose,

    Done,
}

impl State {
    fn wants(
        self,
        seq: u64,
        addr: &SocketAddrUnix,
        writebuf: &[u8],
        readbuf: &mut [u8],
    ) -> (Self, Option<Wants>) {
        match self {
            Self::ReadyToSocket => (
                Self::WaitingForSocket,
                Some(Wants::Socket {
                    domain: AddressFamily::UNIX,
                    r#type: SocketType::STREAM,
                    seq,
                }),
            ),

            Self::ReadyToConnect { fd } => (
                Self::WaitingForConnect { fd },
                Some(Wants::Connect {
                    fd,
                    addr: addr.clone().into(),
                    seq,
                }),
            ),

            Self::ReadyToWrite { fd } => (
                Self::WaitingForWrite { fd },
                Some(Wants::Write {
                    fd,
                    buf: writebuf.as_ptr(),
                    len: writebuf.len(),
                    seq,
                }),
            ),

            Self::ReadyToRead { fd } => (
                Self::WaitingForRead { fd },
                Some(Wants::Read {
                    fd,
                    buf: readbuf.as_mut_ptr(),
                    len: readbuf.len(),
                    seq,
                }),
            ),

            Self::ReadyToClose { fd } => (Self::WaitingForClose, Some(Wants::Close { fd, seq })),

            waiting => (waiting, None),
        }
    }

    fn wants_in_place(
        &mut self,
        seq: u64,
        addr: &SocketAddrUnix,
        writebuf: &[u8],
        readbuf: &mut [u8],
    ) -> Option<Wants> {
        let mut this = Self::Done;
        std::mem::swap(self, &mut this);
        let (next, wants) = this.wants(seq, addr, writebuf, readbuf);
        *self = next;
        wants
    }
}

impl UnixSocketOneshotWriter {
    pub(crate) fn new(addr: SocketAddrUnix, data: &str) -> Result<Self> {
        let mut writebuf = [0; 4_096];
        let mut writer = ArrayWriter::new(&mut writebuf);
        write!(&mut writer, "{data}").context("failed to write command to buffer")?;
        let writebuflen = writer.offset;

        Ok(Self {
            addr,
            writebuf,
            writebuflen,
            readbuf: [0; _],
            readbuflen: 0,
            state: State::ReadyToSocket,
            seq: 0,
        })
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        self.state.wants_in_place(
            self.seq,
            &self.addr,
            self.writebuf
                .get(..self.writebuflen)
                .unwrap_or_else(|| unreachable!()),
            &mut self.readbuf,
        )
    }

    pub(crate) fn satisfy_socket(&mut self, fd: BorrowedFd<'static>) -> Result<()> {
        ensure!(
            matches!(self.state, State::WaitingForSocket),
            "malformed state: expected Socket, got {:?}",
            self.state
        );

        self.state = State::ReadyToConnect { fd };
        self.seq += 1;
        Ok(())
    }

    pub(crate) fn satisfy_connect(&mut self) -> Result<()> {
        let State::WaitingForConnect { fd } = self.state else {
            bail!("malformed state: expected Connect, got {:?}", self.state)
        };

        self.state = State::ReadyToWrite { fd };
        self.seq += 1;
        Ok(())
    }

    pub(crate) fn satisfy_write(&mut self) -> Result<()> {
        let State::WaitingForWrite { fd } = self.state else {
            bail!("malformed state: expected Write, got {:?}", self.state)
        };

        self.state = State::ReadyToRead { fd };
        self.seq += 1;
        Ok(())
    }

    #[expect(dead_code)]
    pub(crate) fn satisfy_read(&mut self, bytes_read: usize) -> Result<()> {
        let State::WaitingForRead { fd } = self.state else {
            bail!("malformed state: expected Read, got {:?}", self.state)
        };

        self.readbuflen = bytes_read;
        self.state = State::ReadyToClose { fd };
        self.seq += 1;
        Ok(())
    }

    #[expect(dead_code)]
    pub(crate) fn satisfy_close(&mut self) -> Result<&[u8]> {
        let State::WaitingForClose = self.state else {
            bail!("malformed state: expected Close, got {:?}", self.state)
        };

        self.seq += 1;
        self.readbuf
            .get(..self.readbuflen)
            .context("buf is too short")
    }

    pub(crate) fn fd(&self) -> Result<BorrowedFd<'static>> {
        match self.state {
            State::ReadyToConnect { fd }
            | State::WaitingForConnect { fd }
            | State::ReadyToWrite { fd }
            | State::WaitingForWrite { fd }
            | State::ReadyToRead { fd }
            | State::WaitingForRead { fd }
            | State::ReadyToClose { fd } => Ok(fd),

            State::ReadyToSocket
            | State::WaitingForSocket
            | State::WaitingForClose
            | State::Done => bail!(
                "UnixSocketOneshotWriter doesn't have FD in {:?} state",
                self.state
            ),
        }
    }
}
