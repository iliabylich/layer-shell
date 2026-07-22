use crate::{
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

impl FileReader {
    pub(crate) fn new(path: &'static CStr) -> Result<Self, ()> {
        log::trace!("Creating FileReader");

        let fd = rustix::fs::open(path, OFlags::RDONLY, Mode::empty()).map_err(|errno| {
            log::error!("failed to open: {errno:?}");
        })?;
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
    ) -> Result<Option<&'a [u8]>, ()> {
        if let (State::WaitingForRead, Satisfy::Read(res)) = (self.state, satisfy) {
            let bytes_read = res?;
            let Some(out) = buf.get(..bytes_read) else {
                log_err_and_exit!(
                    "FileReader: buffer is too short: {bytes_read} vs {}",
                    buf.len()
                )
            };
            self.state = State::Sleeping;
            Ok(Some(out))
        } else {
            log::error!("wrong satisfy {satisfy:?} for {self:?}");
            Err(())
        }
    }

    pub(crate) const fn tick(&mut self) {
        if matches!(self.state, State::Sleeping) {
            self.state = State::CanRead;
        }
    }
}
