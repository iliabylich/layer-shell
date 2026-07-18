use crate::sansio::{Satisfy, Wants};
use anyhow::{Context as _, Result, bail};
use core::ffi::CStr;
use libc::{AT_FDCWD, O_RDONLY};

pub(crate) struct FileReader {
    fd: i32,
    state: State,
}

#[derive(Debug, Clone, Copy)]
enum State {
    CanRead,
    WaitingForRead,
    Sleeping,
}

impl FileReader {
    pub(crate) fn new(path: &'static CStr) -> Result<Self> {
        let res = unsafe { libc::openat(AT_FDCWD, path.as_ptr(), O_RDONLY) };
        if res < 0 {
            bail!("failed to open {path:?}");
        }

        Ok(Self {
            fd: res,
            state: State::CanRead,
        })
    }

    pub(crate) const fn wants(&mut self, buf: &mut [u8]) -> Option<Wants> {
        match self.state {
            State::CanRead => {
                self.state = State::WaitingForRead;
                Some(Wants::Read {
                    fd: self.fd,
                    buf: buf.as_mut_ptr(),
                    len: buf.len(),
                })
            }

            State::WaitingForRead | State::Sleeping => None,
        }
    }

    pub(crate) fn try_satisfy<'a>(
        &mut self,
        satisfy: Satisfy,
        buf: &'a [u8],
    ) -> Result<Option<&'a [u8]>> {
        match (self.state, satisfy) {
            (State::WaitingForRead, Satisfy::Read(bytes_read)) => {
                let bytes_read = bytes_read?;
                let out = buf.get(..bytes_read).context("buffer is too short")?;
                self.state = State::Sleeping;
                Ok(Some(out))
            }

            (state, satisfy) => {
                bail!("malformed FileReader state: {state:?} for {satisfy:?}");
            }
        }
    }

    pub(crate) const fn tick(&mut self) {
        if matches!(self.state, State::Sleeping) {
            self.state = State::CanRead;
        }
    }
}
