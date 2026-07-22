use crate::{
    error::IoError,
    sansio::{Satisfy, Wants},
    utils::log_err_and_exit,
};
use core::ffi::CStr;
use rustix::{
    fd::{BorrowedFd, IntoRawFd},
    fs::{Mode, OFlags},
};

#[derive(Debug, Clone, Copy)]
pub struct FileReader {
    fd: BorrowedFd<'static>,
    state: State,
}

#[derive(Debug, Clone, Copy)]
enum State {
    CanRead,
    WaitingForRead,
    Sleeping,
}
impl State {
    const fn as_str(self) -> &'static str {
        match self {
            Self::CanRead => "CanRead",
            Self::WaitingForRead => "WaitingForRead",
            Self::Sleeping => "Sleeping",
        }
    }
}

impl FileReader {
    pub(crate) fn new(path: &'static CStr) -> Result<Self, IoError> {
        let fd = rustix::fs::open(path, OFlags::RDONLY, Mode::empty())
            .map_err(|errno| IoError::FailedTo { op: "open", errno })?;
        let fd = unsafe { BorrowedFd::borrow_raw(fd.into_raw_fd()) };

        Ok(Self {
            fd,
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
    ) -> Result<Option<&'a [u8]>, IoError> {
        match (self.state, satisfy) {
            (State::WaitingForRead, Satisfy::Read(res)) => {
                let bytes_read = res?;
                let Some(out) = buf.get(..bytes_read) else {
                    log_err_and_exit!(
                        "FileReader: buffer is too short: {} vs {}",
                        bytes_read,
                        buf.len()
                    )
                };
                self.state = State::Sleeping;
                Ok(Some(out))
            }

            _ => Err(IoError::WrongSatisfy {
                state: self.state.as_str(),
                satisfy: satisfy.as_str(),
            }),
        }
    }

    pub(crate) const fn tick(&mut self) {
        if matches!(self.state, State::Sleeping) {
            self.state = State::CanRead;
        }
    }
}
