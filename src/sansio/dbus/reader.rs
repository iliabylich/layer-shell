use crate::sansio::{Satisfy, Wants};
use anyhow::{Result, bail, ensure};

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
const HEADER_LEN: usize = std::mem::size_of::<Header>();

pub(crate) struct DBusReader {
    fd: i32,
    bytes_read: usize,
    message_len: usize,
    state: State,
    buf: Box<[u8; BUF_SIZE]>,
}

#[derive(Debug, Clone, Copy)]
enum State {
    CanReadHeader,
    WaitingForHeader,
    CanReadBody,
    WaitingForBody,
}

const BUF_SIZE: usize = 500_000;

impl DBusReader {
    pub(crate) fn new(fd: i32) -> Self {
        Self {
            fd,
            bytes_read: 0,
            message_len: 0,
            state: State::CanReadHeader,
            buf: Box::new([0; BUF_SIZE]),
        }
    }

    pub(crate) fn wants(&mut self) -> Wants {
        match self.state {
            State::CanReadHeader => {
                self.state = State::WaitingForHeader;

                Wants::Read {
                    fd: self.fd,
                    buf: self.buf.as_mut_ptr(),
                    len: HEADER_LEN,
                }
            }
            State::WaitingForHeader => Wants::Nothing,

            State::CanReadBody => {
                let buf = &mut self.buf[self.bytes_read..self.message_len];
                self.state = State::WaitingForBody;
                Wants::Read {
                    fd: self.fd,
                    buf: buf.as_mut_ptr(),
                    len: buf.len(),
                }
            }
            State::WaitingForBody => Wants::Nothing,
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<&[u8]>> {
        match (self.state, satisfy) {
            (State::WaitingForHeader, Satisfy::Read) => {
                if res == 0 {
                    return Ok(None);
                }
                ensure!(res > 0);
                let bytes_read = res as usize;
                ensure!(bytes_read == HEADER_LEN);
                self.bytes_read += bytes_read;

                let header = unsafe { &*self.buf.as_ptr().cast::<Header>() };

                self.message_len = HEADER_LEN
                    + (header.header_fields_len as usize).next_multiple_of(8)
                    + header.body_len as usize;

                self.state = State::CanReadBody;

                Ok(None)
            }

            (State::WaitingForBody, Satisfy::Read) => {
                ensure!(res > 0);
                let bytes_read = res as usize;
                self.bytes_read += bytes_read;

                if self.bytes_read == self.message_len {
                    let message_len = self.message_len;

                    self.bytes_read = 0;
                    self.message_len = 0;
                    self.state = State::CanReadHeader;

                    return Ok(Some(&self.buf[..message_len]));
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
}
