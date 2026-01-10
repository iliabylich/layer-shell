use crate::{
    dbus::{Message, encoders::MessageEncoder, serial::Serial},
    liburing::IoUring,
    user_data::UserData,
};
use anyhow::{Result, ensure};
use std::{collections::VecDeque, os::fd::AsRawFd};

#[derive(Debug, Default)]
enum State {
    #[default]
    CanWriteZero,
    WritingZero,

    CanWriteAuthExternal,
    WritingAuthExternal,

    CanReadData,
    ReadingData,

    CanWriteData,
    WritingData,

    CanReadGUID,
    ReadingGUID,

    CanWriteBegin,
    WritingBegin,

    Finished,
}

#[derive(Debug)]
pub(crate) struct Auth {
    fd: i32,
    state: State,
    buf: [u8; 100],
    queue: VecDeque<Vec<u8>>,
    serial: Serial,

    write_zero_user_data: UserData,
    write_auth_external_user_data: UserData,
    read_data_user_data: UserData,
    write_data_user_data: UserData,
    read_guid_user_data: UserData,
    write_begin_user_data: UserData,

    pub(crate) read_header_user_data: UserData,
    pub(crate) read_body_user_data: UserData,
    pub(crate) write_user_data: UserData,
}

const AUTH_EXTERNAL: &[u8] = b"AUTH EXTERNAL\r\n";
const DATA: &[u8] = b"DATA\r\n";
const BEGIN: &[u8] = b"BEGIN\r\n";

impl Auth {
    pub(crate) fn new(
        fd: i32,
        write_zero_user_data: UserData,
        write_auth_external_user_data: UserData,
        read_data_user_data: UserData,
        write_data_user_data: UserData,
        read_guid_user_data: UserData,
        write_begin_user_data: UserData,
        read_header_user_data: UserData,
        read_body_user_data: UserData,
        write_user_data: UserData,
    ) -> Self {
        Self {
            fd,
            state: State::default(),
            buf: [0; 100],
            queue: VecDeque::new(),
            serial: Serial::zero(),

            write_zero_user_data,
            write_auth_external_user_data,
            read_data_user_data,
            write_data_user_data,
            read_guid_user_data,
            write_begin_user_data,

            read_header_user_data,
            read_body_user_data,
            write_user_data,
        }
    }

    pub(crate) fn enqueue(&mut self, message: &mut Message) -> Result<()> {
        *message.serial_mut() = self.serial.increment_and_get();
        let bytes = MessageEncoder::encode(message)?;
        self.queue.push_back(bytes);
        Ok(())
    }

    pub(crate) fn drain(&mut self, ring: &mut IoUring) -> Result<bool> {
        match self.state {
            State::CanWriteZero => {
                let mut sqe = ring.get_sqe()?;
                sqe.prep_write(self.fd, c"".as_ptr().cast(), 1);
                sqe.set_user_data(self.write_zero_user_data.as_u64());
                self.state = State::WritingZero;
                Ok(true)
            }
            State::WritingZero => Ok(false),

            State::CanWriteAuthExternal => {
                let mut sqe = ring.get_sqe()?;
                sqe.prep_write(self.fd, AUTH_EXTERNAL.as_ptr(), AUTH_EXTERNAL.len());
                sqe.set_user_data(self.write_auth_external_user_data.as_u64());
                self.state = State::WritingAuthExternal;
                Ok(true)
            }
            State::WritingAuthExternal => Ok(false),

            State::CanReadData => {
                let mut sqe = ring.get_sqe()?;
                sqe.prep_read(self.fd, self.buf.as_mut_ptr(), self.buf.len());
                sqe.set_user_data(self.read_data_user_data.as_u64());
                self.state = State::ReadingData;
                Ok(true)
            }
            State::ReadingData => Ok(false),

            State::CanWriteData => {
                let mut sqe = ring.get_sqe()?;
                sqe.prep_write(self.fd, DATA.as_ptr(), DATA.len());
                sqe.set_user_data(self.write_data_user_data.as_u64());
                self.state = State::WritingData;
                Ok(true)
            }
            State::WritingData => Ok(false),

            State::CanReadGUID => {
                let mut sqe = ring.get_sqe()?;
                sqe.prep_read(self.fd, self.buf.as_mut_ptr(), self.buf.len());
                sqe.set_user_data(self.read_guid_user_data.as_u64());
                self.state = State::ReadingGUID;
                Ok(true)
            }
            State::ReadingGUID => Ok(false),

            State::CanWriteBegin => {
                let mut sqe = ring.get_sqe()?;
                sqe.prep_write(self.fd, BEGIN.as_ptr(), BEGIN.len());
                sqe.set_user_data(self.write_begin_user_data.as_u64());
                self.state = State::WritingBegin;
                Ok(true)
            }
            State::WritingBegin => Ok(false),

            State::Finished => Ok(false),
        }
    }

    pub(crate) fn feed(&mut self, user_data: UserData, res: i32) -> Result<bool> {
        if user_data == self.write_zero_user_data {
            ensure!(
                matches!(self.state, State::WritingZero),
                "malformed state, expected WritingZero, got {:?}",
                self.state
            );

            assert!(res > 0);
            let written = res as usize;
            assert_eq!(written, 1);
            self.state = State::CanWriteAuthExternal;
            return Ok(false);
        }

        if user_data == self.write_auth_external_user_data {
            ensure!(
                matches!(self.state, State::WritingAuthExternal),
                "malformed state, expected WritingAuthExternal, got {:?}",
                self.state
            );

            assert!(res > 0);
            let written = res as usize;
            assert_eq!(written, AUTH_EXTERNAL.len());
            self.state = State::CanReadData;
            return Ok(false);
        }

        if user_data == self.read_data_user_data {
            ensure!(
                matches!(self.state, State::ReadingData),
                "malformed state, expected ReadingData, got {:?}",
                self.state
            );

            assert!(res > 0);
            let read = res as usize;
            assert_eq!(read, DATA.len());
            assert_eq!(&self.buf[..read], DATA);
            self.state = State::CanWriteData;
            return Ok(false);
        }

        if user_data == self.write_data_user_data {
            ensure!(
                matches!(self.state, State::WritingData),
                "malformed state, expected WritingData, got {:?}",
                self.state
            );

            assert!(res > 0);
            let written = res as usize;
            assert_eq!(written, DATA.len());
            self.state = State::CanReadGUID;
            return Ok(false);
        }

        if user_data == self.read_guid_user_data {
            ensure!(
                matches!(self.state, State::ReadingGUID),
                "malformed state, expected ReadingGUID, got {:?}",
                self.state
            );

            assert!(res > 0);
            self.state = State::CanWriteBegin;
            return Ok(false);
        }

        if user_data == self.write_begin_user_data {
            ensure!(
                matches!(self.state, State::WritingBegin),
                "malformed state, expected WritingBegin, got {:?}",
                self.state
            );

            assert!(res > 0);
            let written = res as usize;
            assert_eq!(written, BEGIN.len());
            self.state = State::Finished;
            return Ok(true);
        }

        Ok(false)
    }

    pub(crate) fn take_queue(&mut self) -> VecDeque<Vec<u8>> {
        std::mem::take(&mut self.queue)
    }

    pub(crate) fn take_serial(&mut self) -> Serial {
        std::mem::take(&mut self.serial)
    }
}

impl AsRawFd for Auth {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        self.fd
    }
}
