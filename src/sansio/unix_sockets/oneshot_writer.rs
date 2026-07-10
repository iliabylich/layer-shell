use crate::{
    external::{__socket_type_SOCK_STREAM as SOCK_STREAM, AF_UNIX, sockaddr_un, socklen_t},
    sansio::Wants,
    utils::ArrayWriter,
};
use anyhow::{Context, Result, bail, ensure};
use core::{fmt::Write, mem::size_of};

pub(crate) struct UnixSocketOneshotWriter {
    writebuf: [u8; 4_096],
    writebuflen: usize,
    readbuf: [u8; 4_096],
    readbuflen: usize,
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

    ReadyToRead { fd: i32 },
    WaitingForRead { fd: i32 },

    ReadyToClose { fd: i32 },
    WaitingForClose,

    Done,
}

impl State {
    const fn wants(
        self,
        addr: &sockaddr_un,
        writebuf: &[u8],
        readbuf: &mut [u8],
    ) -> (Self, Option<Wants>) {
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
                    addrlen: size_of::<sockaddr_un>() as socklen_t,
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

            Self::ReadyToRead { fd } => (
                Self::WaitingForRead { fd },
                Some(Wants::Read {
                    fd,
                    buf: readbuf.as_mut_ptr(),
                    len: readbuf.len(),
                }),
            ),

            Self::ReadyToClose { fd } => (Self::WaitingForClose, Some(Wants::Close { fd })),

            waiting => (waiting, None),
        }
    }

    const fn wants_in_place(
        &mut self,
        addr: &sockaddr_un,
        writebuf: &[u8],
        readbuf: &mut [u8],
    ) -> Option<Wants> {
        let mut this = Self::Done;
        core::mem::swap(self, &mut this);
        let (next, wants) = this.wants(addr, writebuf, readbuf);
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
            readbuf: [0; _],
            readbuflen: 0,
            state: State::ReadyToSocket,
        })
    }

    pub(crate) fn wants(&mut self, addr: &sockaddr_un) -> Option<Wants> {
        self.state.wants_in_place(
            addr,
            self.writebuf
                .get(..self.writebuflen)
                .unwrap_or_else(|| unreachable!()),
            &mut self.readbuf,
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

    pub(crate) fn satisfy_write(&mut self) -> Result<()> {
        let State::WaitingForWrite { fd } = self.state else {
            bail!("malformed state: expected Write, got {:?}", self.state)
        };

        self.state = State::ReadyToRead { fd };
        Ok(())
    }

    #[expect(dead_code)]
    pub(crate) fn satisfy_read(&mut self, bytes_read: usize) -> Result<()> {
        let State::WaitingForRead { fd } = self.state else {
            bail!("malformed state: expected Read, got {:?}", self.state)
        };

        self.readbuflen = bytes_read;
        self.state = State::ReadyToClose { fd };
        Ok(())
    }

    #[expect(dead_code)]
    pub(crate) fn satisfy_close(&self) -> Result<&[u8]> {
        let State::WaitingForClose = self.state else {
            bail!("malformed state: expected Close, got {:?}", self.state)
        };

        self.readbuf
            .get(..self.readbuflen)
            .context("buf is too short")
    }

    pub(crate) fn fd(&self) -> Result<i32> {
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
