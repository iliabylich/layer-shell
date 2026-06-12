use crate::sansio::Wants;
use anyhow::{Context as _, Result, bail, ensure};
use rustix::fs::{CWD, Mode, OFlags};
use std::os::fd::BorrowedFd;

pub(crate) struct FileReader<const N: usize> {
    path: &'static str,
    state: State,
    buf: Box<[u8; N]>,
    seq: u64,
}

#[derive(Debug)]
enum State {
    CanOpen,
    CanRead { fd: BorrowedFd<'static> },
    Sleeping { fd: BorrowedFd<'static> },
}

impl<const N: usize> FileReader<N> {
    pub(crate) fn new(path: &'static str) -> Self {
        Self {
            path,
            state: State::CanOpen,
            buf: Box::new([0; N]),
            seq: 0,
        }
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        match self.state {
            State::CanOpen => Some(Wants::OpenAt {
                dfd: CWD,
                path: self.path,
                flags: OFlags::RDONLY,
                mode: Mode::empty(),
                seq: self.seq,
            }),

            State::CanRead { fd } => Some(Wants::Read {
                fd,
                buf: self.buf.as_mut_ptr(),
                len: self.buf.len(),
                seq: self.seq,
            }),

            State::Sleeping { .. } => None,
        }
    }

    pub(crate) fn satisfy_open(&mut self, fd: BorrowedFd<'static>) -> Result<()> {
        ensure!(
            matches!(self.state, State::CanOpen),
            "malformed state: expected Open, got {:?}",
            self.state
        );

        self.state = State::CanRead { fd };
        self.seq += 1;
        Ok(())
    }

    pub(crate) fn satisfy_read(&mut self, bytes_read: usize) -> Result<&[u8]> {
        let State::CanRead { fd } = self.state else {
            bail!("malformed state: expected Read, got {:?}", self.state)
        };

        let out = self.buf.get(..bytes_read).context("buffer is too short")?;
        self.state = State::Sleeping { fd };
        self.seq += 1;
        Ok(out)
    }

    pub(crate) const fn satisfy_tick(&mut self) {
        if let State::Sleeping { fd } = self.state {
            self.state = State::CanRead { fd };
            self.seq += 1;
        }
    }
}
