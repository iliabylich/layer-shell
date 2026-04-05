use crate::sansio::{Satisfy, Wants};
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
    Dead,
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

            State::Dead => Wants::Nothing,
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Option<&[u8]> {
        match (self.state, satisfy) {
            (State::Dead, _) => None,
            (_, Satisfy::Crash) => {
                log::error!("Module FileReader received Satisfy::Crash, stopping...");
                self.state = State::Dead;
                None
            }

            (State::WaitingForOpen, Satisfy::OpenAt) => {
                if res < 0 {
                    log::error!("FileReader::Open failed: {res}");
                    self.state = State::Dead;
                    return None;
                }

                self.fd = res;
                self.state = State::CanRead;
                None
            }

            (State::WaitingForRead, Satisfy::Read) => {
                if res <= 0 {
                    log::error!("FileReader::Read failed: {res}");
                    self.state = State::Dead;
                    return None;
                }

                let bytes_read = res as usize;
                let out = &self.buf[..bytes_read];
                self.state = State::WaitingForTimer;
                Some(out)
            }

            (state, satisfy) => {
                log::error!("malformed FileReader state: {state:?} vs {satisfy:?}");
                self.state = State::Dead;
                None
            }
        }
    }

    pub(crate) fn tick(&mut self) {
        if matches!(self.state, State::WaitingForTimer) {
            self.state = State::CanRead;
        }
    }
}
