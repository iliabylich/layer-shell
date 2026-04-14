use crate::sansio::{Satisfy, Wants};
use anyhow::{Result, bail, ensure};
use libc::{AT_FDCWD, O_RDONLY};
use std::ffi::CStr;

pub(crate) struct FileReader {
    path: &'static CStr,
    fd: i32,
    state: State,
}

#[derive(Debug, Clone, Copy)]
enum State {
    CanOpen,
    WaitingForOpen,
    WaitingForTimer,
    CanRead,
    WaitingForRead,
    Dead,
}

const BUF_SIZE: usize = 1_024;
static mut BUFFER: Option<Box<[u8; BUF_SIZE]>> = None;
fn buffer() -> &'static mut [u8; BUF_SIZE] {
    unsafe {
        if BUFFER.is_none() {
            BUFFER = Some(Box::new([0; BUF_SIZE]));
        }

        BUFFER.as_mut().unwrap_unchecked()
    }
}

impl FileReader {
    pub(crate) fn new(path: &'static CStr) -> Self {
        Self {
            path,
            fd: -1,
            state: State::CanOpen,
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
                    buf: buffer().as_mut_ptr(),
                    len: buffer().len(),
                }
            }
            State::WaitingForRead => Wants::Nothing,

            State::Dead => Wants::Nothing,
        }
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<&'static [u8]>> {
        match (self.state, satisfy) {
            (State::Dead, _) => Ok(None),

            (State::WaitingForOpen, Satisfy::OpenAt) => {
                ensure!(res >= 0, "FileReader::Open failed: {res}");
                self.fd = res;
                self.state = State::CanRead;
                Ok(None)
            }

            (State::WaitingForRead, Satisfy::Read) => {
                ensure!(res > 0, "FileReader::Read failed: {res}");
                let bytes_read = res as usize;
                let out = &buffer()[..bytes_read];
                self.state = State::WaitingForTimer;
                Ok(Some(out))
            }

            (state, satisfy) => {
                bail!("malformed FileReader state: {state:?} vs {satisfy:?}");
            }
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Option<&'static [u8]> {
        match self.try_satisfy(satisfy, res) {
            Ok(bytes) => bytes,
            Err(err) => {
                log::error!("FileReader got an error: {err:?}");
                self.stop();
                None
            }
        }
    }

    pub(crate) fn tick(&mut self) {
        if matches!(self.state, State::WaitingForTimer) {
            self.state = State::CanRead;
        }
    }

    pub(crate) fn stop(&mut self) {
        log::error!("Stopping FileReader");
        self.state = State::Dead;
    }
}
