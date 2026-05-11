use crate::sansio::{Satisfy, Wants};
use anyhow::{Context as _, Result, bail, ensure};
use libc::{AT_FDCWD, O_RDONLY};
use std::ffi::CStr;

pub(crate) struct FileReader<const N: usize> {
    path: &'static CStr,
    fd: i32,
    state: State,
    buf: Box<[u8; N]>,
}

#[derive(Debug, Clone, Copy)]
enum State {
    ReadyTo(Action),
    WaitingFor(Action),
    Sleeping,
}

#[derive(Debug, Clone, Copy)]
enum Action {
    Open,
    Read,
}

impl<const N: usize> FileReader<N> {
    pub(crate) fn new(path: &'static CStr) -> Self {
        Self {
            path,
            fd: -1,
            state: State::ReadyTo(Action::Open),
            buf: Box::new([0; N]),
        }
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        match self.state {
            State::ReadyTo(Action::Open) => {
                self.state = State::WaitingFor(Action::Open);
                Some(Wants::OpenAt {
                    dfd: AT_FDCWD,
                    path: self.path.as_ptr(),
                    flags: O_RDONLY,
                    mode: 0,
                    seq: 42,
                })
            }

            State::ReadyTo(Action::Read) => {
                self.state = State::WaitingFor(Action::Read);
                Some(Wants::Read {
                    fd: self.fd,
                    buf: self.buf.as_mut_ptr(),
                    len: self.buf.len(),
                    seq: 42,
                })
            }

            State::WaitingFor(_) | State::Sleeping => None,
        }
    }

    pub(crate) fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<&[u8]>> {
        match (self.state, satisfy) {
            (State::WaitingFor(Action::Open), Satisfy::OpenAt) => {
                ensure!(res >= 0);
                self.fd = res;
                self.state = State::ReadyTo(Action::Read);
                Ok(None)
            }

            (State::WaitingFor(Action::Read), Satisfy::Read) => {
                let bytes_read = usize::try_from(res).context("read failed")?;
                let out = self.buf.get(..bytes_read).context("buffer is too short")?;
                self.state = State::Sleeping;
                Ok(Some(out))
            }

            _ => {
                bail!("malformed state {:?}", self.state);
            }
        }
    }

    pub(crate) const fn tick(&mut self) {
        if matches!(self.state, State::Sleeping) {
            self.state = State::ReadyTo(Action::Read);
        }
    }
}
