use crate::external::{
    BIO_CTRL_PENDING, BIO_ctrl, BIO_read, BIO_write, SSL_ERROR_WANT_READ, SSL_ERROR_WANT_WRITE,
    SSL_ERROR_ZERO_RETURN, SSL_get_error, SSL_read, SSL_write,
};
use crate::sansio::{Satisfy, Wants, https::OpenSslState};
use alloc::{boxed::Box, vec, vec::Vec};
use anyhow::{Context as _, Result, bail, ensure};

#[derive(Debug, Clone, Copy)]
enum State {
    ReadyToRead,
    WaitingForRead,

    ReadyToWrite,
    WaitingForWrite,
}

pub(crate) struct OpenSslReadWrite {
    state: State,

    tls: OpenSslState,

    readbuf: Box<[u8; 4_096]>,
    writebuf: Vec<u8>,

    request: Vec<u8>,
    response: Vec<u8>,
}

impl OpenSslReadWrite {
    pub(crate) fn new(ssl: OpenSslState, request: Vec<u8>) -> Result<Self> {
        let mut this = Self {
            state: State::ReadyToWrite,

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
                i32::try_from(self.request.len()).context("request len doesn't fit into i32")?,
            )
        };
        if res <= 0 {
            let err = unsafe { SSL_get_error(self.tls.ssl, res) };
            self.drain_wbio()?;
            if !self.writebuf.is_empty() {
                self.state = State::ReadyToWrite;
                return Ok(());
            }

            match err {
                SSL_ERROR_WANT_READ => {
                    self.state = State::ReadyToRead;
                    return Ok(());
                }
                SSL_ERROR_WANT_WRITE => {
                    bail!("OpenSslReadWrite: SSL_write wanted write with empty wbio");
                }
                _ => bail!("OpenSslReadWrite: SSL_write failed {err}"),
            }
        }

        let bytes_written = usize::try_from(res).context("SSL_write failed")?;
        ensure!(
            bytes_written == self.request.len(),
            "OpenSslReadWrite: SSL_write failed {bytes_written} != {}",
            self.request.len()
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
        self.writebuf.clear();
        while unsafe { BIO_ctrl(self.tls.wbio, BIO_CTRL_PENDING, 0, core::ptr::null_mut()) } > 0 {
            let mut buf = [0_u8; 1_024];
            let read = unsafe { BIO_read(self.tls.wbio, buf.as_mut_ptr().cast(), 1_024) };
            let len = usize::try_from(read).context("OpenSslReadWrite: BIO_read failed")?;
            self.writebuf
                .extend_from_slice(buf.get(..len).context("buffer is too short")?);
        }
        Ok(())
    }

    pub(crate) fn wants(&mut self, fd: i32) -> Option<Wants> {
        match self.state {
            State::ReadyToRead => {
                self.state = State::WaitingForRead;
                Some(Wants::Read {
                    fd,
                    buf: self.readbuf.as_mut_ptr(),
                    len: self.readbuf.len(),
                })
            }
            State::ReadyToWrite => {
                self.state = State::WaitingForWrite;
                Some(Wants::Write {
                    fd,
                    buf: self.writebuf.as_ptr(),
                    len: self.writebuf.len(),
                })
            }

            State::WaitingForRead | State::WaitingForWrite => None,
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy) -> Result<Option<Vec<u8>>> {
        match (self.state, satisfy) {
            (State::WaitingForWrite, Satisfy::Write(res)) => {
                let bytes_written = res?;
                ensure!(
                    bytes_written == self.writebuf.len(),
                    "OpenSslReadWrite: write failed {bytes_written} != {}",
                    self.writebuf.len()
                );
                self.writebuf.clear();
                self.state = State::ReadyToRead;
                Ok(None)
            }

            (State::WaitingForRead, Satisfy::Read(res)) => {
                let bytes_read = res?;
                ensure!(bytes_read > 0, "OpenSslReadWrite: EOF");
                let encrypted = self.readbuf.get(..bytes_read).context("buf is too short")?;
                let bytes_read = i32::try_from(bytes_read).unwrap_or_else(|_| unreachable!());
                let written =
                    unsafe { BIO_write(self.tls.rbio, encrypted.as_ptr().cast(), bytes_read) };
                ensure!(
                    written == bytes_read,
                    "OpenSslReadWrite: BIO_write failed: {written} != {bytes_read}"
                );

                let mut plaintext = [0_u8; 1_024];
                loop {
                    let res =
                        unsafe { SSL_read(self.tls.ssl, plaintext.as_mut_ptr().cast(), 1_024) };
                    if let Ok(res) = usize::try_from(res)
                        && res > 0
                    {
                        self.response.extend_from_slice(
                            plaintext
                                .get(..res)
                                .context("plaintext buffer is too short")?,
                        );
                    } else {
                        let err = unsafe { SSL_get_error(self.tls.ssl, res) };

                        match err {
                            SSL_ERROR_WANT_READ => {
                                self.state = State::ReadyToRead;
                                return Ok(None);
                            }

                            SSL_ERROR_WANT_WRITE => {
                                self.drain_wbio()?;
                                ensure!(
                                    !self.writebuf.is_empty(),
                                    "OpenSslReadWrite: writebuf is empty after draining wbio"
                                );
                                self.state = State::ReadyToWrite;
                                return Ok(None);
                            }

                            SSL_ERROR_ZERO_RETURN => {
                                self.tls.free();
                                return Ok(Some(core::mem::take(&mut self.response)));
                            }

                            _ => {
                                bail!("OpenSslReadWrite: SSL_read failed {err}");
                            }
                        }
                    }
                }
            }

            (_, satisfy) => bail!(
                "OpenSslReadWrite: wrong Satisfy {satisfy:?} for state {:?}",
                self.state
            ),
        }
    }
}
