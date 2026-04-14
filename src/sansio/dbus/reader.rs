use crate::sansio::{DBusConnectionKind, Satisfy, Wants};
use anyhow::{Context as _, Result, bail, ensure};

#[repr(C, packed)]
struct Header {
    _endian: u8,
    _message_type: u8,
    _flags: u8,
    _protocol_version: u8,
    body_len: u32,
    _serial: u32,
    header_fields_len: u32,
}
const HEADER_LEN: usize = core::mem::size_of::<Header>();

pub(crate) struct DBusReader {
    fd: i32,
    bytes_read: usize,
    message_len: usize,
    discard_remaining: usize,
    state: State,
    kind: DBusConnectionKind,
}

#[derive(Debug, Clone, Copy)]
enum State {
    CanReadHeader,
    WaitingForHeader,
    CanReadBody,
    WaitingForBody,
    CanDiscardBody,
    WaitingForDiscardBody,
    Dead,
}

const BUF_SIZE: usize = 500_000;
static mut BUFFERS: Option<Vec<[u8; BUF_SIZE]>> = None;
fn buffer(kind: DBusConnectionKind) -> &'static mut [u8; BUF_SIZE] {
    unsafe {
        if BUFFERS.is_none() {
            BUFFERS = Some(vec![[0; BUF_SIZE], [0; BUF_SIZE]]);
        }

        BUFFERS
            .as_mut()
            .unwrap_unchecked()
            .get_mut(kind as usize)
            .unwrap_unchecked()
    }
}

impl DBusReader {
    pub(crate) fn new(fd: i32, kind: DBusConnectionKind) -> Self {
        Self {
            fd,
            bytes_read: 0,
            message_len: 0,
            discard_remaining: 0,
            state: State::CanReadHeader,
            kind,
        }
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        match self.state {
            State::CanReadHeader => {
                self.state = State::WaitingForHeader;

                Some(Wants::Read {
                    fd: self.fd,
                    buf: buffer(self.kind).as_mut_ptr(),
                    len: HEADER_LEN,
                })
            }
            State::WaitingForHeader => None,

            State::CanReadBody => {
                let buf = &mut buffer(self.kind)[self.bytes_read..self.message_len];
                self.state = State::WaitingForBody;
                Some(Wants::Read {
                    fd: self.fd,
                    buf: buf.as_mut_ptr(),
                    len: buf.len(),
                })
            }
            State::WaitingForBody => None,

            State::CanDiscardBody => {
                let len = self.discard_remaining.min(BUF_SIZE);
                self.state = State::WaitingForDiscardBody;
                Some(Wants::Read {
                    fd: self.fd,
                    buf: buffer(self.kind).as_mut_ptr(),
                    len,
                })
            }
            State::WaitingForDiscardBody => None,

            State::Dead => None,
        }
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<&'static [u8]>> {
        match (self.state, satisfy) {
            (State::Dead, _) => Ok(None),

            (State::WaitingForHeader, Satisfy::Read) => {
                if res == 0 {
                    return Ok(None);
                }
                ensure!(res > 0, "DBusReader::ReadHeader failed: {res}");
                let bytes_read = res as usize;
                ensure!(bytes_read == HEADER_LEN);
                self.bytes_read += bytes_read;

                let header = unsafe { &*buffer(self.kind).as_ptr().cast::<Header>() };

                let header_fields_len = (header.header_fields_len as usize).next_multiple_of(8);
                let message_len = HEADER_LEN
                    .checked_add(header_fields_len)
                    .and_then(|len| len.checked_add(header.body_len as usize))
                    .context("dbus message length overflow")?;
                if message_len <= BUF_SIZE {
                    self.message_len = message_len;
                    self.state = State::CanReadBody;
                } else {
                    self.discard_remaining = message_len - HEADER_LEN;
                    self.state = State::CanDiscardBody;
                }

                Ok(None)
            }

            (State::WaitingForDiscardBody, Satisfy::Read) => {
                ensure!(res > 0, "DBusReader::DiscardBody failed: {res}");
                let bytes_read = res as usize;
                ensure!(
                    bytes_read <= self.discard_remaining,
                    "dbus discard read overflow: {} > {}",
                    bytes_read,
                    self.discard_remaining
                );

                self.discard_remaining -= bytes_read;
                if self.discard_remaining == 0 {
                    self.bytes_read = 0;
                    self.message_len = 0;
                    self.discard_remaining = 0;
                    self.state = State::CanReadHeader;
                } else {
                    self.state = State::CanDiscardBody;
                }

                Ok(None)
            }

            (State::WaitingForBody, Satisfy::Read) => {
                ensure!(res > 0, "DBusReader::ReadBody failed: {res}");
                let bytes_read = res as usize;
                self.bytes_read += bytes_read;

                if self.bytes_read == self.message_len {
                    let message_len = self.message_len;

                    self.bytes_read = 0;
                    self.message_len = 0;
                    self.state = State::CanReadHeader;

                    return Ok(Some(&buffer(self.kind)[..message_len]));
                } else {
                    self.state = State::CanReadBody;
                }

                Ok(None)
            }

            (state, satisfy) => {
                bail!("malformed DBusReader state: {state:?} vs {satisfy:?}")
            }
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Option<&'static [u8]> {
        match self.try_satisfy(satisfy, res) {
            Ok(buf) => buf,
            Err(err) => {
                log::error!("Module DBusReader has crashed: {err:?}");
                self.stop();
                None
            }
        }
    }

    pub(crate) fn stop(&mut self) {
        log::error!("Stopping DBusReader({:?})", self.kind);
        self.state = State::Dead;
    }
}
