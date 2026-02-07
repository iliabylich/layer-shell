use crate::{
    liburing::IoUring,
    user_data::{ModuleId, UserData},
};
use anyhow::{Result, ensure};

#[repr(u8)]
enum Op {
    Write,
}
const MAX_OP: u8 = Op::Write as u8;

impl TryFrom<u8> for Op {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self> {
        ensure!(value <= MAX_OP);
        unsafe { Ok(std::mem::transmute::<u8, Self>(value)) }
    }
}

pub(crate) struct Writer {
    fd: i32,
    module_id: ModuleId,
    buf: Vec<u8>,
}

impl Writer {
    pub(crate) fn new(fd: i32, module_id: ModuleId) -> Self {
        Self {
            fd,
            module_id,
            buf: vec![],
        }
    }

    pub(crate) fn init(&mut self, buf: Vec<u8>) {
        self.buf = buf;
        let mut sqe = IoUring::get_sqe();
        sqe.prep_write(self.fd, self.buf.as_ptr(), self.buf.len());
        sqe.set_user_data(UserData::new(self.module_id, Op::Write as u8));
    }

    pub(crate) fn process(&self, op: u8, res: i32) -> Result<()> {
        match Op::try_from(op)? {
            Op::Write => {
                ensure!(res > 0);
                let written = res as usize;
                ensure!(written == self.buf.len());
            }
        }
        Ok(())
    }
}
