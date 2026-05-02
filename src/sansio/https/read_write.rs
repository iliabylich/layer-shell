use crate::sansio::{Satisfy, Wants, https::OpenSslState};
use anyhow::{Result, bail, ensure};
use openssl_sys::{
    BIO_ctrl, BIO_read, BIO_write, SSL_ERROR_WANT_READ, SSL_ERROR_WANT_WRITE,
    SSL_ERROR_ZERO_RETURN, SSL_get_error, SSL_read, SSL_write,
};
use std::rc::Rc;

#[derive(Debug, Clone, Copy)]
enum State {
    ReadyToRead,
    WaitingForRead,

    ReadyToWrite,
    WaitingForWrite,
}

pub(crate) struct OpenSslReadWrite {
    state: State,
    fd: i32,

    tls: Rc<OpenSslState>,

    readbuf: Box<[u8; 4_096]>,
    writebuf: Vec<u8>,

    request: Vec<u8>,
    response: Vec<u8>,
}

impl OpenSslReadWrite {
    pub(crate) fn new(fd: i32, ssl: Rc<OpenSslState>, request: Vec<u8>) -> Result<Self> {
        let mut this = Self {
            state: State::ReadyToWrite,
            fd,

            tls: ssl,

            readbuf: Box::new([0; _]),
            writebuf: vec![],

            request,
            response: vec![],
        };
        this.write_request()?;
        Ok(this)
    }

    fn write_request(&mut self) -> Result<()> {
        let res = unsafe {
            SSL_write(
                self.tls.ssl,
                self.request.as_ptr().cast(),
                self.request.len() as i32,
            )
        };
        ensure!(
            res == self.request.len() as i32,
            "OpenSslReadWrite: SSL_write failed {res}"
        );

        self.drain_wbio()?;
        ensure!(
            !self.writebuf.is_empty(),
            "OpenSslReadWrite: writebuf is empty after draining wbio"
        );
        self.state = State::ReadyToWrite;
        Ok(())
    }

    fn drain_wbio(&mut self) -> Result<()> {
        const BIO_CTRL_PENDING: i32 = 10;
        self.writebuf.clear();
        while unsafe { BIO_ctrl(self.tls.wbio, BIO_CTRL_PENDING, 0, std::ptr::null_mut()) } > 0 {
            let mut buf = [0_u8; 1_024];
            let read = unsafe { BIO_read(self.tls.wbio, buf.as_mut_ptr().cast(), 1_024) };
            ensure!(read > 0, "OpenSslReadWrite: {read} <= 0");
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

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<Vec<u8>>> {
        match (self.state, satisfy) {
            (State::WaitingForWrite, Satisfy::Write) => {
                ensure!(res > 0, "OpenSslReadWrite: write failed {res}");
                self.writebuf.clear();
                self.state = State::ReadyToRead;
                Ok(None)
            }

            (State::WaitingForRead, Satisfy::Read) => {
                ensure!(res > 0, "OpenSslReadWrite: read failed {res}");
                let encrypted = &self.readbuf[..res as usize];
                let written = unsafe { BIO_write(self.tls.rbio, encrypted.as_ptr().cast(), res) };
                ensure!(
                    written == res,
                    "OpenSslReadWrite: BIO_write failed: {written} != {res}"
                );

                let mut plaintext = [0_u8; 1_024];
                loop {
                    let res = unsafe {
                        SSL_read(
                            self.tls.ssl,
                            plaintext.as_mut_ptr().cast(),
                            plaintext.len() as i32,
                        )
                    };
                    if res > 0 {
                        self.response.extend_from_slice(&plaintext[..res as usize]);
                    } else {
                        let err = unsafe { SSL_get_error(self.tls.ssl, res) };
                        if err == SSL_ERROR_WANT_READ {
                            self.state = State::ReadyToRead;
                            return Ok(None);
                        } else if err == SSL_ERROR_WANT_WRITE {
                            self.drain_wbio()?;
                            ensure!(
                                !self.writebuf.is_empty(),
                                "OpenSslReadWrite: writebuf is empty after draining wbio"
                            );
                            self.state = State::ReadyToWrite;
                            return Ok(None);
                        } else if err == SSL_ERROR_ZERO_RETURN {
                            return Ok(Some(std::mem::take(&mut self.response)));
                        } else {
                            bail!("OpenSslReadWrite: SSL_read failed {err}");
                        }
                    }
                }
            }

            _ => bail!(
                "OpenSslReadWrite: wrong Satisfy {satisfy:?} for state {:?}",
                self.state
            ),
        }
    }
}
