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
    ReadyTo(Action),
    WaitingFor(Action),
    Dead,
}

#[derive(Debug, Clone, Copy)]
enum Action {
    ReadHeader,
    ReadBody,
    Discard,
}

impl DBusReader {
    pub(crate) fn new(fd: i32, kind: DBusConnectionKind) -> Self {
        Self {
            fd,
            bytes_read: 0,
            message_len: 0,
            discard_remaining: 0,
            state: State::ReadyTo(Action::ReadHeader),
            kind,
        }
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        let State::ReadyTo(action) = self.state else {
            return None;
        };

        let wants = match action {
            Action::ReadHeader => Wants::Read {
                fd: self.fd,
                buf: self.kind.read_buffer().as_mut_ptr(),
                len: HEADER_LEN,
            },

            Action::ReadBody => {
                let buf = &mut self.kind.read_buffer()[self.bytes_read..self.message_len];
                Wants::Read {
                    fd: self.fd,
                    buf: buf.as_mut_ptr(),
                    len: buf.len(),
                }
            }

            Action::Discard => {
                let len = self
                    .discard_remaining
                    .min(DBusConnectionKind::READ_BUF_SIZE);
                Wants::Read {
                    fd: self.fd,
                    buf: self.kind.read_buffer().as_mut_ptr(),
                    len,
                }
            }
        };
        self.state = State::WaitingFor(action);
        Some(wants)
    }

    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<&'static [u8]>> {
        let action = match self.state {
            State::WaitingFor(action) => action,
            State::Dead => return Ok(None),
            state => bail!("malformed DBusReader state: {state:?} vs {satisfy:?}"),
        };

        match (action, satisfy) {
            (Action::ReadHeader, Satisfy::Read) => {
                if res == 0 {
                    return Ok(None);
                }
                ensure!(res > 0, "DBusReader::ReadHeader failed: {res}");
                let bytes_read = res as usize;
                ensure!(bytes_read == HEADER_LEN);
                self.bytes_read += bytes_read;

                let header = unsafe { &*self.kind.read_buffer().as_ptr().cast::<Header>() };

                let header_fields_len = (header.header_fields_len as usize).next_multiple_of(8);
                let message_len = HEADER_LEN
                    .checked_add(header_fields_len)
                    .and_then(|len| len.checked_add(header.body_len as usize))
                    .context("dbus message length overflow")?;
                if message_len <= DBusConnectionKind::READ_BUF_SIZE {
                    self.message_len = message_len;
                    self.state = State::ReadyTo(Action::ReadBody);
                } else {
                    self.discard_remaining = message_len - HEADER_LEN;
                    self.state = State::ReadyTo(Action::Discard);
                }

                Ok(None)
            }

            (Action::Discard, Satisfy::Read) => {
                ensure!(res > 0, "DBusReader::DiscardBody failed: {res}");
                let bytes_read = res as usize;
                ensure!(
                    bytes_read <= self.discard_remaining,
                    "dbus discard read overflow: {bytes_read} > {}",
                    self.discard_remaining
                );

                self.discard_remaining -= bytes_read;
                if self.discard_remaining == 0 {
                    self.bytes_read = 0;
                    self.message_len = 0;
                    self.discard_remaining = 0;
                    self.state = State::ReadyTo(Action::ReadHeader);
                } else {
                    self.state = State::ReadyTo(Action::Discard);
                }

                Ok(None)
            }

            (Action::ReadBody, Satisfy::Read) => {
                ensure!(res > 0, "DBusReader::ReadBody failed: {res}");
                let bytes_read = res as usize;
                self.bytes_read += bytes_read;

                if self.bytes_read == self.message_len {
                    let message_len = self.message_len;

                    self.bytes_read = 0;
                    self.message_len = 0;
                    self.state = State::ReadyTo(Action::ReadHeader);

                    return Ok(Some(&self.kind.read_buffer()[..message_len]));
                } else {
                    self.state = State::ReadyTo(Action::ReadBody);
                }

                Ok(None)
            }

            (_, _) => {
                bail!("malformed DBusReader state: {action:?} vs {satisfy:?}")
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
