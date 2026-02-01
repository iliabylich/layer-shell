use crate::{
    dbus::{Message, encoders::MessageEncoder, serial::Serial},
    liburing::IoUring,
    user_data::{ModuleId, UserData},
};
use anyhow::{Result, ensure};
use std::collections::VecDeque;

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
    pub(crate) module_id: ModuleId,
}

const AUTH_EXTERNAL: &[u8] = b"AUTH EXTERNAL\r\n";
const DATA: &[u8] = b"DATA\r\n";
const BEGIN: &[u8] = b"BEGIN\r\n";

#[repr(u8)]
enum Op {
    WriteZero,
    WriteAuthExternal,
    ReadData,
    WriteData,
    ReadGUID,
    WriteBegin,
}
const MAX_OP: u8 = Op::WriteBegin as u8;

impl TryFrom<u8> for Op {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self> {
        ensure!(value <= MAX_OP);
        unsafe { Ok(std::mem::transmute::<u8, Self>(value)) }
    }
}

impl Auth {
    pub(crate) fn new(fd: i32, module_id: ModuleId) -> Self {
        Self {
            fd,
            state: State::default(),
            buf: [0; 100],
            queue: VecDeque::new(),
            serial: Serial::zero(),
            module_id,
        }
    }

    pub(crate) fn enqueue(&mut self, message: &mut Message) {
        *message.serial_mut() = self.serial.increment_and_get();
        let bytes = MessageEncoder::encode(message);
        self.queue.push_back(bytes);
    }

    pub(crate) fn drain(&mut self, ring: &mut IoUring) -> Result<()> {
        match self.state {
            State::CanWriteZero => {
                let mut sqe = ring.get_sqe()?;
                sqe.prep_write(self.fd, c"".as_ptr().cast(), 1);
                sqe.set_user_data(UserData::new(self.module_id, Op::WriteZero as u8));
                self.state = State::WritingZero;
                Ok(())
            }
            State::WritingZero => Ok(()),

            State::CanWriteAuthExternal => {
                let mut sqe = ring.get_sqe()?;
                sqe.prep_write(self.fd, AUTH_EXTERNAL.as_ptr(), AUTH_EXTERNAL.len());
                sqe.set_user_data(UserData::new(self.module_id, Op::WriteAuthExternal as u8));
                self.state = State::WritingAuthExternal;
                Ok(())
            }
            State::WritingAuthExternal => Ok(()),

            State::CanReadData => {
                let mut sqe = ring.get_sqe()?;
                sqe.prep_read(self.fd, self.buf.as_mut_ptr(), self.buf.len());
                sqe.set_user_data(UserData::new(self.module_id, Op::ReadData as u8));
                self.state = State::ReadingData;
                Ok(())
            }
            State::ReadingData => Ok(()),

            State::CanWriteData => {
                let mut sqe = ring.get_sqe()?;
                sqe.prep_write(self.fd, DATA.as_ptr(), DATA.len());
                sqe.set_user_data(UserData::new(self.module_id, Op::WriteData as u8));
                self.state = State::WritingData;
                Ok(())
            }
            State::WritingData => Ok(()),

            State::CanReadGUID => {
                let mut sqe = ring.get_sqe()?;
                sqe.prep_read(self.fd, self.buf.as_mut_ptr(), self.buf.len());
                sqe.set_user_data(UserData::new(self.module_id, Op::ReadGUID as u8));
                self.state = State::ReadingGUID;
                Ok(())
            }
            State::ReadingGUID => Ok(()),

            State::CanWriteBegin => {
                let mut sqe = ring.get_sqe()?;
                sqe.prep_write(self.fd, BEGIN.as_ptr(), BEGIN.len());
                sqe.set_user_data(UserData::new(self.module_id, Op::WriteBegin as u8));
                self.state = State::WritingBegin;
                Ok(())
            }
            State::WritingBegin => Ok(()),

            State::Finished => Ok(()),
        }
    }

    pub(crate) fn feed(
        &mut self,
        op: u8,
        res: i32,
    ) -> Result<Option<(i32, Serial, VecDeque<Vec<u8>>)>> {
        match Op::try_from(op)? {
            Op::WriteZero => {
                assert!(res > 0);
                let written = res as usize;
                assert_eq!(written, 1);
                self.state = State::CanWriteAuthExternal;
                Ok(None)
            }
            Op::WriteAuthExternal => {
                assert!(res > 0);
                let written = res as usize;
                assert_eq!(written, AUTH_EXTERNAL.len());
                self.state = State::CanReadData;
                Ok(None)
            }
            Op::ReadData => {
                assert!(res > 0);
                let read = res as usize;
                assert_eq!(read, DATA.len());
                assert_eq!(&self.buf[..read], DATA);
                self.state = State::CanWriteData;
                Ok(None)
            }
            Op::WriteData => {
                assert!(res > 0);
                let written = res as usize;
                assert_eq!(written, DATA.len());
                self.state = State::CanReadGUID;
                Ok(None)
            }
            Op::ReadGUID => {
                assert!(res > 0);
                self.state = State::CanWriteBegin;
                Ok(None)
            }
            Op::WriteBegin => {
                assert!(res > 0);
                let written = res as usize;
                assert_eq!(written, BEGIN.len());
                self.state = State::Finished;

                let fd = self.fd;
                let serial = std::mem::take(&mut self.serial);
                let queue = std::mem::take(&mut self.queue);
                Ok(Some((fd, serial, queue)))
            }
        }
    }
}
