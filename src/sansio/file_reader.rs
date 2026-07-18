use crate::sansio::{Satisfy, Wants};
use anyhow::{Context as _, Result, bail};
use core::ffi::CStr;
use libc::{AT_FDCWD, O_RDONLY};

pub(crate) struct FileReader {
    path: &'static CStr,
    state: State,
}

#[derive(Debug, Clone, Copy)]
enum State {
    CanOpen,
    WaitingForOpen,
    CanRead { fd: i32 },
    WaitingForRead { fd: i32 },
    Sleeping { fd: i32 },
}

impl FileReader {
    pub(crate) const fn new(path: &'static CStr) -> Self {
        Self {
            path,
            state: State::CanOpen,
        }
    }

    pub(crate) const fn wants(&mut self, buf: &mut [u8]) -> Option<Wants> {
        match self.state {
            State::CanOpen => {
                self.state = State::WaitingForOpen;
                Some(Wants::OpenAt {
                    dfd: AT_FDCWD,
                    path: self.path,
                    flags: O_RDONLY,
                    mode: 0,
                })
            }

            State::CanRead { fd } => {
                self.state = State::WaitingForRead { fd };
                Some(Wants::Read {
                    fd,
                    buf: buf.as_mut_ptr(),
                    len: buf.len(),
                })
            }

            State::WaitingForOpen | State::WaitingForRead { .. } | State::Sleeping { .. } => None,
        }
    }

    pub(crate) fn try_satisfy<'a>(
        &mut self,
        satisfy: Satisfy,
        buf: &'a [u8],
    ) -> Result<Option<&'a [u8]>> {
        match (self.state, satisfy) {
            (State::WaitingForOpen, Satisfy::OpenAt(fd)) => {
                let fd = fd?;
                self.state = State::CanRead { fd };
                Ok(None)
            }

            (State::WaitingForRead { fd }, Satisfy::Read(bytes_read)) => {
                let bytes_read = bytes_read?;
                let out = buf.get(..bytes_read).context("buffer is too short")?;
                self.state = State::Sleeping { fd };
                Ok(Some(out))
            }

            (state, satisfy) => {
                bail!("malformed FileReader state: {state:?} for {satisfy:?}");
            }
        }
    }

    pub(crate) const fn tick(&mut self) {
        if let State::Sleeping { fd } = self.state {
            self.state = State::CanRead { fd };
        }
    }
}
