use crate::sansio::{Satisfy, Wants};
use anyhow::{Result, bail, ensure};
use libc::{AT_FDCWD, O_RDONLY};
use std::ffi::CStr;

pub(crate) struct FileReader {
    path: &'static CStr,
    fd: i32,
    state: State,
    buf: [u8; 1_024],
}

#[derive(Debug, Clone, Copy)]
enum State {
    CanOpen,
    WaitingForOpen,
    WaitingForTimer,
    CanRead,
    WaitingForRead,
}

impl FileReader {
    pub(crate) fn new(path: &'static CStr) -> Self {
        Self {
            path,
            fd: -1,
            state: State::CanOpen,
            buf: [0; _],
        }
    }

    pub(crate) fn wants(&mut self) -> Wants {
        match self.state {
            State::CanOpen => {
                self.state = State::WaitingForOpen;
                Wants::OpenAt {
                    dfd: AT_FDCWD,
                    path: self.path.as_ptr(),
                    flags: O_RDONLY,
                    mode: 0,
                }
            }
            State::WaitingForOpen => Wants::Nothing,

            State::WaitingForTimer => Wants::Nothing,

            State::CanRead => {
                self.state = State::WaitingForRead;
                Wants::Read {
                    fd: self.fd,
                    buf: self.buf.as_mut_ptr(),
                    len: self.buf.len(),
                }
            }
            State::WaitingForRead => Wants::Nothing,
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<&[u8]>> {
        match (self.state, satisfy) {
            (State::WaitingForOpen, Satisfy::OpenAt) => {
                ensure!(res >= 0);
                self.fd = res;
                self.state = State::CanRead;
                Ok(None)
            }

            (State::WaitingForRead, Satisfy::Read) => {
                ensure!(res > 0);
                let bytes_read = res as usize;
                let out = &self.buf[..bytes_read];
                self.state = State::WaitingForTimer;
                Ok(Some(out))
            }

            (state, satisfy) => {
                bail!("malformed FileReader state: {state:?} vs {satisfy:?}")
            }
        }
    }

    pub(crate) fn tick(&mut self) {
        if matches!(self.state, State::WaitingForTimer) {
            self.state = State::CanRead;
        }
    }
}
