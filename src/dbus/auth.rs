use crate::{
    dbus::ConnectionKind,
    liburing::IoUring,
    macros::define_op,
    user_data::{ModuleId, UserData},
};

define_op!(
    "DBus Auth",
    WriteZero,
    WriteAuthExternal,
    ReadData,
    WriteData,
    ReadGUID,
    WriteBegin,
);

#[derive(Debug)]
pub(crate) struct Auth {
    kind: ConnectionKind,
    fd: i32,
    buf: [u8; 100],
    module_id: ModuleId,
    healthy: bool,
}

const AUTH_EXTERNAL: &[u8] = b"AUTH EXTERNAL\r\n";
const DATA: &[u8] = b"DATA\r\n";
const BEGIN: &[u8] = b"BEGIN\r\n";

impl Auth {
    pub(crate) fn new(kind: ConnectionKind) -> Self {
        let module_id = match kind {
            ConnectionKind::Session => ModuleId::SessionDBusAuth,
            ConnectionKind::System => ModuleId::SystemDBusAuth,
        };

        Self {
            kind,
            fd: -1,
            buf: [0; 100],
            module_id,
            healthy: true,
        }
    }

    fn schedule_write_zero(&self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_write(self.fd, c"".as_ptr().cast(), 1);
        sqe.set_user_data(UserData::new(self.module_id, Op::WriteZero));
    }

    fn schedule_write_auth_external(&self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_write(self.fd, AUTH_EXTERNAL.as_ptr(), AUTH_EXTERNAL.len());
        sqe.set_user_data(UserData::new(self.module_id, Op::WriteAuthExternal));
    }

    fn schedule_read_data(&mut self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_read(self.fd, self.buf.as_mut_ptr(), self.buf.len());
        sqe.set_user_data(UserData::new(self.module_id, Op::ReadData));
    }

    fn schedule_write_data(&self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_write(self.fd, DATA.as_ptr(), DATA.len());
        sqe.set_user_data(UserData::new(self.module_id, Op::WriteData));
    }

    fn schedule_read_guid(&mut self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_read(self.fd, self.buf.as_mut_ptr(), self.buf.len());
        sqe.set_user_data(UserData::new(self.module_id, Op::ReadGUID));
    }

    fn schedule_write_begin(&mut self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_write(self.fd, BEGIN.as_ptr(), BEGIN.len());
        sqe.set_user_data(UserData::new(self.module_id, Op::WriteBegin));
    }

    pub(crate) fn init(&mut self, fd: i32) {
        self.fd = fd;
        self.schedule_write_zero()
    }

    pub(crate) fn process(&mut self, op: u8, res: i32) -> bool {
        if !self.healthy {
            return false;
        }

        let op = Op::from(op);

        macro_rules! assert_eq_or_unhealthy {
            ($actual:expr, $expected:expr) => {
                if $actual != $expected {
                    log::error!(
                        "DBusAuth({:?})::{op:?}: actual ({:?}) != expected ({:?})",
                        self.kind,
                        $actual,
                        $expected
                    );
                    self.healthy = false;
                    return false;
                }
            };
        }

        match op {
            Op::WriteZero => {
                let written = res as usize;
                assert_eq_or_unhealthy!(written, 1);
                self.schedule_write_auth_external();
                false
            }
            Op::WriteAuthExternal => {
                let written = res as usize;
                assert_eq_or_unhealthy!(written, AUTH_EXTERNAL.len());
                self.schedule_read_data();
                false
            }
            Op::ReadData => {
                let read = res as usize;
                assert_eq_or_unhealthy!(read, DATA.len());
                assert_eq_or_unhealthy!(&self.buf[..read], DATA);
                self.schedule_write_data();
                false
            }
            Op::WriteData => {
                let written = res as usize;
                assert_eq_or_unhealthy!(written, DATA.len());
                self.schedule_read_guid();
                false
            }
            Op::ReadGUID => {
                self.schedule_write_begin();
                false
            }
            Op::WriteBegin => {
                let written = res as usize;
                assert_eq_or_unhealthy!(written, BEGIN.len());
                true
            }
        }
    }
}
