use crate::{
    liburing::IoUring,
    user_data::{ModuleId, UserData},
};
use anyhow::{Result, ensure};

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

#[derive(Debug)]
pub(crate) struct Auth {
    fd: i32,
    buf: [u8; 100],
    module_id: ModuleId,
}

const AUTH_EXTERNAL: &[u8] = b"AUTH EXTERNAL\r\n";
const DATA: &[u8] = b"DATA\r\n";
const BEGIN: &[u8] = b"BEGIN\r\n";

impl Auth {
    pub(crate) fn new(fd: i32, module_id: ModuleId) -> Self {
        Self {
            fd,
            buf: [0; 100],
            module_id,
        }
    }

    fn schedule_write_zero(&self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_write(self.fd, c"".as_ptr().cast(), 1);
        sqe.set_user_data(UserData::new(self.module_id, Op::WriteZero as u8));
    }

    fn schedule_write_auth_external(&self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_write(self.fd, AUTH_EXTERNAL.as_ptr(), AUTH_EXTERNAL.len());
        sqe.set_user_data(UserData::new(self.module_id, Op::WriteAuthExternal as u8));
    }

    fn schedule_read_data(&mut self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_read(self.fd, self.buf.as_mut_ptr(), self.buf.len());
        sqe.set_user_data(UserData::new(self.module_id, Op::ReadData as u8));
    }

    fn schedule_write_data(&self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_write(self.fd, DATA.as_ptr(), DATA.len());
        sqe.set_user_data(UserData::new(self.module_id, Op::WriteData as u8));
    }

    fn schedule_read_guid(&mut self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_read(self.fd, self.buf.as_mut_ptr(), self.buf.len());
        sqe.set_user_data(UserData::new(self.module_id, Op::ReadGUID as u8));
    }

    fn schedule_write_begin(&mut self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_write(self.fd, BEGIN.as_ptr(), BEGIN.len());
        sqe.set_user_data(UserData::new(self.module_id, Op::WriteBegin as u8));
    }

    pub(crate) fn init(&mut self) {
        self.schedule_write_zero()
    }

    pub(crate) fn process(&mut self, op: u8, res: i32) -> Result<bool> {
        match Op::try_from(op)? {
            Op::WriteZero => {
                ensure!(res > 0);
                let written = res as usize;
                ensure!(written == 1);
                self.schedule_write_auth_external();
                Ok(false)
            }
            Op::WriteAuthExternal => {
                ensure!(res > 0);
                let written = res as usize;
                ensure!(written == AUTH_EXTERNAL.len());
                self.schedule_read_data();
                Ok(false)
            }
            Op::ReadData => {
                ensure!(res > 0);
                let read = res as usize;
                ensure!(read == DATA.len());
                ensure!(&self.buf[..read] == DATA);
                self.schedule_write_data();
                Ok(false)
            }
            Op::WriteData => {
                ensure!(res > 0);
                let written = res as usize;
                ensure!(written == DATA.len());
                self.schedule_read_guid();
                Ok(false)
            }
            Op::ReadGUID => {
                ensure!(res > 0);
                self.schedule_write_begin();
                Ok(false)
            }
            Op::WriteBegin => {
                ensure!(res > 0);
                let written = res as usize;
                ensure!(written == BEGIN.len());
                Ok(true)
            }
        }
    }
}
