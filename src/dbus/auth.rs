use crate::{
    liburing::IoUring,
    macros::report_and_exit,
    user_data::{ModuleId, UserData},
};

#[repr(u8)]
#[derive(Debug)]
enum Op {
    WriteZero,
    WriteAuthExternal,
    ReadData,
    WriteData,
    ReadGUID,
    WriteBegin,
}
const MAX_OP: u8 = Op::WriteBegin as u8;

impl From<u8> for Op {
    fn from(value: u8) -> Self {
        if value > MAX_OP {
            report_and_exit!("unsupported op in DBus Auth: {value}");
        }
        unsafe { std::mem::transmute::<u8, Self>(value) }
    }
}

#[derive(Debug)]
pub(crate) struct Auth {
    fd: i32,
    buf: [u8; 100],
    module_id: ModuleId,
    healthy: bool,
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
            healthy: true,
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

    pub(crate) fn process(&mut self, op: u8, res: i32) -> bool {
        if !self.healthy {
            return false;
        }

        macro_rules! crash {
            ($($arg:tt)*) => {{
                log::error!($($arg)*);
                self.healthy = false;
                return false;
            }};
        }

        let op = Op::from(op);

        if res <= 0 {
            crash!("{op:?} returned {res}");
        }

        match op {
            Op::WriteZero => {
                let written = res as usize;
                if written != 1 {
                    crash!("{op:?} returned {written} bytes (expected 1)");
                }
                self.schedule_write_auth_external();
                false
            }
            Op::WriteAuthExternal => {
                let written = res as usize;
                if written != AUTH_EXTERNAL.len() {
                    crash!(
                        "{op:?} returned {written} bytes (expected {})",
                        AUTH_EXTERNAL.len()
                    );
                }
                self.schedule_read_data();
                false
            }
            Op::ReadData => {
                let read = res as usize;
                if read != DATA.len() {
                    crash!("{op:?} returned {read} bytes (expected {})", DATA.len());
                }
                if &self.buf[..read] != DATA {
                    crash!(
                        "{op:?} returned {:?} (expected {:?})",
                        &self.buf[..read],
                        DATA
                    );
                }
                self.schedule_write_data();
                false
            }
            Op::WriteData => {
                let written = res as usize;
                if written != DATA.len() {
                    crash!("{op:?} returned {written} bytes (expected {})", DATA.len());
                }
                self.schedule_read_guid();
                false
            }
            Op::ReadGUID => {
                self.schedule_write_begin();
                false
            }
            Op::WriteBegin => {
                let written = res as usize;
                if written != BEGIN.len() {
                    crash!("{op:?} returned {written} bytes (expected {})", BEGIN.len());
                }
                true
            }
        }
    }
}
