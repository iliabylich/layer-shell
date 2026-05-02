use crate::sansio::{Satisfy, Wants, https::state::OpenSslState};
use anyhow::{Result, bail, ensure};
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

        self.drain_wbio()?;
        if !self.writebuf.is_empty() {
            return Ok(Progress::NextState(State::ReadyToWrite));
        }

        if ret == 1 {
            return Ok(Progress::Done);
        }

        let err = unsafe { SSL_get_error(self.tls.ssl, ret) };

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
            let read = unsafe { BIO_read(self.tls.wbio, buf.as_mut_ptr().cast(), 1_024) };
            ensure!(read > 0, "OpenSslHandshake: BIO_read failed: {read} <= 0");
            self.writebuf.extend_from_slice(&buf[..read as usize]);
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
                })
            }

            State::ReadyToWrite => {
                self.state = State::WaitingForWrite;
                Some(Wants::Write {
                    fd: self.fd,
                    buf: self.writebuf.as_ptr(),
                    len: self.writebuf.len(),
                })
            }

            State::WaitingForRead | State::WaitingForWrite => None,
        }
    }

    pub(crate) fn satisfy(
        &mut self,
        satisfy: Satisfy,
        res: i32,
    ) -> Result<Option<Rc<OpenSslState>>> {
        match (self.state, satisfy) {
            (State::WaitingForRead, Satisfy::Read) => {
                ensure!(res > 0, "OpenSslHandshake: read failed {res}");
                let received = &self.readbuf[..res as usize];
                let written = unsafe {
                    BIO_write(
                        self.tls.rbio,
                        received.as_ptr().cast(),
                        received.len() as i32,
                    )
                };
                ensure!(
                    written == res,
                    "OpenSslHandshake: read failed {written} != {res}"
                );
            }

            (State::WaitingForWrite, Satisfy::Write) => {
                ensure!(res > 0, "OpenSslHandshake: write failed {res}");
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
