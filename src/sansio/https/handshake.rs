use crate::sansio::{Satisfy, Wants, https::state::OpenSslState};
use anyhow::{Context as _, Result, bail, ensure};
use openssl_sys::{
    BIO_ctrl, BIO_read, BIO_write, SSL_ERROR_WANT_READ, SSL_ERROR_WANT_WRITE, SSL_connect,
    SSL_get_error,
};
use std::rc::Rc;

#[derive(Debug, Clone, Copy)]
enum State {
    ReadyToRead,
    WaitingForRead,

    ReadyToWrite,
    WaitingForWrite,
}

pub(crate) struct OpenSslHandshake {
    state: State,
    fd: i32,

    tls: Rc<OpenSslState>,

    readbuf: Box<[u8; 4_096]>,
    writebuf: Vec<u8>,
}

enum Progress {
    Done,
    NextState(State),
}

impl OpenSslHandshake {
    pub(crate) fn new(fd: i32, hostname: &str) -> Result<Self> {
        let mut this = Self {
            state: State::ReadyToRead,
            fd,

            tls: OpenSslState::new(hostname)?,

            readbuf: Box::new([0; _]),
            writebuf: vec![],
        };
        this.state = match this.determine_state()? {
            Progress::Done => bail!("bug: there must be state at the beginning"),
            Progress::NextState(state) => state,
        };
        Ok(this)
    }

    fn determine_state(&mut self) -> Result<Progress> {
        let ret = unsafe { SSL_connect(self.tls.ssl) };

        if ret == 1 {
            self.drain_wbio()?;
            if !self.writebuf.is_empty() {
                return Ok(Progress::NextState(State::ReadyToWrite));
            }
            return Ok(Progress::Done);
        }

        let err = unsafe { SSL_get_error(self.tls.ssl, ret) };
        self.drain_wbio()?;
        if !self.writebuf.is_empty() {
            return Ok(Progress::NextState(State::ReadyToWrite));
        }

        if err == SSL_ERROR_WANT_READ {
            Ok(Progress::NextState(State::ReadyToRead))
        } else if err == SSL_ERROR_WANT_WRITE {
            Ok(Progress::NextState(State::ReadyToWrite))
        } else {
            bail!("OpenSslHandshake: SSL_connect failed {err}");
        }
    }

    fn drain_wbio(&mut self) -> Result<()> {
        const BIO_CTRL_PENDING: i32 = 10;
        self.writebuf.clear();
        while unsafe { BIO_ctrl(self.tls.wbio, BIO_CTRL_PENDING, 0, std::ptr::null_mut()) } > 0 {
            let mut buf = [0_u8; 1_024];
            let res = unsafe { BIO_read(self.tls.wbio, buf.as_mut_ptr().cast(), 1_024) };
            let bytes_read = usize::try_from(res).context("BIO_read failed")?;
            self.writebuf
                .extend_from_slice(buf.get(..bytes_read).context("buf is too short")?);
        }
        Ok(())
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        match self.state {
            State::ReadyToRead => {
                self.state = State::WaitingForRead;
                Some(Wants::Read {
                    fd: self.fd,
                    buf: self.readbuf.as_mut_ptr(),
                    len: self.readbuf.len(),
                    seq: 42,
                })
            }

            State::ReadyToWrite => {
                self.state = State::WaitingForWrite;
                Some(Wants::Write {
                    fd: self.fd,
                    buf: self.writebuf.as_ptr(),
                    len: self.writebuf.len(),
                    seq: 42,
                })
            }

            State::WaitingForRead | State::WaitingForWrite => None,
        }
    }

    pub(crate) const fn is_waiting(&self) -> bool {
        matches!(self.state, State::WaitingForRead | State::WaitingForWrite)
    }

    pub(crate) fn satisfy(
        &mut self,
        satisfy: Satisfy,
        res: i32,
    ) -> Result<Option<Rc<OpenSslState>>> {
        match (self.state, satisfy) {
            (State::WaitingForRead, Satisfy::Read) => {
                let bytes_read = usize::try_from(res).context("read failed")?;
                ensure!(bytes_read > 0, "OpenSslHandshake: EOF");
                let received = self
                    .readbuf
                    .get(..bytes_read)
                    .context("readbuf is too short")?;
                let written = unsafe { BIO_write(self.tls.rbio, received.as_ptr().cast(), res) };
                ensure!(
                    written == res,
                    "OpenSslHandshake: read failed {written} != {res}"
                );
            }

            (State::WaitingForWrite, Satisfy::Write) => {
                let bytes_written =
                    usize::try_from(res).context("OpenSslHandshake: write failed")?;
                ensure!(
                    bytes_written == self.writebuf.len(),
                    "OpenSslHandshake: write failed {bytes_written} != {}",
                    self.writebuf.len()
                );
            }

            _ => bail!(
                "OpenSslHandshake: wrong Satisfy {satisfy:?} for state {:?}",
                self.state
            ),
        }

        match self.determine_state()? {
            Progress::Done => Ok(Some(Rc::clone(&self.tls))),
            Progress::NextState(state) => {
                self.state = state;
                Ok(None)
            }
        }
    }
}
