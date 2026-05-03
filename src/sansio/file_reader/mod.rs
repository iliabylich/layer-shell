use crate::sansio::{Satisfy, Wants};
use anyhow::{Context as _, Result, bail, ensure};
pub(crate) use kind::FileReaderKind;
use libc::{AT_FDCWD, O_RDONLY};
use std::ffi::CStr;

mod kind;

pub(crate) struct FileReader {
    path: &'static CStr,
    fd: i32,
    state: State,
    kind: FileReaderKind,
}

#[derive(Debug, Clone, Copy)]
enum State {
    ReadyTo(Action),
    WaitingFor(Action),
    Sleeping,
    Dead,
}

#[derive(Debug, Clone, Copy)]
enum Action {
    Open,
    Read,
}

impl FileReader {
    pub(crate) const fn new(path: &'static CStr, kind: FileReaderKind) -> Self {
        Self {
            path,
            fd: -1,
            state: State::ReadyTo(Action::Open),
            kind,
        }
    }

    pub(crate) fn wants(&mut self) -> Result<Option<Wants>> {
        let State::ReadyTo(action) = self.state else {
            return Ok(None);
        };

        let wants = match action {
            Action::Open => Wants::OpenAt {
                dfd: AT_FDCWD,
                path: self.path.as_ptr(),
                flags: O_RDONLY,
                mode: 0,
            },

            Action::Read => Wants::Read {
                fd: self.fd,
                buf: self.kind.buffer()?.as_mut_ptr(),
                len: self.kind.buffer()?.len(),
            },
        };
        self.state = State::WaitingFor(action);
        Ok(Some(wants))
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<&'static [u8]>> {
        let action = match self.state {
            State::WaitingFor(action) => action,
            State::Dead => return Ok(None),
            state => bail!("malformed FileReader state: {state:?} vs {satisfy:?}"),
        };

        match (action, satisfy) {
            (Action::Open, Satisfy::OpenAt) => {
                ensure!(res >= 0, "FileReader::Open failed: {res}");
                self.fd = res;
                self.state = State::ReadyTo(Action::Read);
                Ok(None)
            }

            (Action::Read, Satisfy::Read) => {
                let bytes_read = usize::try_from(res).context("read failed")?;
                let out = self
                    .kind
                    .buffer()?
                    .get(..bytes_read)
                    .context("buffer is too short")?;
                self.state = State::Sleeping;
                Ok(Some(out))
            }

            (state, _) => {
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

    pub(crate) const fn tick(&mut self) {
        if matches!(self.state, State::Sleeping) {
            self.state = State::ReadyTo(Action::Read);
        }
    }

    pub(crate) fn stop(&mut self) {
        log::error!("Stopping FileReader({:?})", self.kind);
        self.state = State::Dead;
    }
}
