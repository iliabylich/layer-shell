use crate::sansio::Wants;
use anyhow::{Context as _, Result, ensure};
use libc::{AT_FDCWD, O_RDONLY};
use std::ffi::CStr;

pub(crate) struct FileReader<const N: usize> {
    path: &'static CStr,
    fd: i32,
    state: State,
    buf: Box<[u8; N]>,
    seq: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Open,
    Read,
    Sleeping,
}

impl<const N: usize> FileReader<N> {
    pub(crate) fn new(path: &'static CStr) -> Self {
        Self {
            path,
            fd: -1,
            state: State::Open,
            buf: Box::new([0; N]),
            seq: 0,
        }
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        match self.state {
            State::Open => Some(Wants::OpenAt {
                dfd: AT_FDCWD,
                path: self.path.as_ptr(),
                flags: O_RDONLY,
                mode: 0,
                seq: self.seq,
            }),

            State::Read => Some(Wants::Read {
                fd: self.fd,
                buf: self.buf.as_mut_ptr(),
                len: self.buf.len(),
                seq: self.seq,
            }),

            State::Sleeping => None,
        }
    }

    pub(crate) fn satisfy_open(&mut self, res: i32) -> Result<()> {
        ensure!(
            self.state == State::Open,
            "malformed state: expected Open, got {:?}",
            self.state
        );

        ensure!(res >= 0);
        self.fd = res;
        self.state = State::Read;
        self.seq += 1;
        Ok(())
    }

    pub(crate) fn satisfy_read(&mut self, res: i32) -> Result<&[u8]> {
        ensure!(
            self.state == State::Read,
            "malformed state: expected Read, got {:?}",
            self.state
        );

        let bytes_read = usize::try_from(res).context("read failed")?;
        let out = self.buf.get(..bytes_read).context("buffer is too short")?;
        self.state = State::Sleeping;
        self.seq += 1;
        Ok(out)
    }

    pub(crate) const fn satisfy_tick(&mut self) {
        if matches!(self.state, State::Sleeping) {
            self.state = State::Read;
            self.seq += 1;
        }
    }
}
