use crate::{
    dbus::ConnectionKind,
    liburing::IoUring,
    macros::define_op,
    user_data::{ModuleId, UserData},
};

define_op!("DBus Writer", Write);

pub(crate) struct Writer {
    kind: ConnectionKind,
    fd: i32,
    module_id: ModuleId,
    buf: Vec<u8>,
    healthy: bool,
}

impl Writer {
    pub(crate) fn new(kind: ConnectionKind) -> Self {
        let module_id = match kind {
            ConnectionKind::Session => ModuleId::SessionDBusWriter,
            ConnectionKind::System => ModuleId::SystemDBusWriter,
        };

        Self {
            kind,
            fd: -1,
            module_id,
            buf: vec![],
            healthy: true,
        }
    }

    fn schedule_write(&mut self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_write(self.fd, self.buf.as_ptr(), self.buf.len());
        sqe.set_user_data(UserData::new(self.module_id, Op::Write));
    }

    pub(crate) fn init(&mut self, fd: i32, buf: Vec<u8>) {
        self.fd = fd;
        self.buf = buf;
        self.schedule_write();
    }

    pub(crate) fn process(&mut self, op: u8, res: i32) {
        if !self.healthy {
            return;
        }

        let op = Op::from(op);

        macro_rules! assert_or_unhealthy {
            ($cond:expr, $($arg:tt)*) => {
                if !$cond {
                    log::error!("DBusWriter({:?})::{op:?}", self.kind);
                    log::error!($($arg)*);
                    self.healthy = false;
                    return;
                }
            };
        }

        match op {
            Op::Write => {
                assert_or_unhealthy!(res >= 0, "res is {res}");
                let written = res as usize;
                assert_or_unhealthy!(
                    written == self.buf.len(),
                    "written is wrong: {written} vs {}",
                    self.buf.len()
                );
            }
        }
    }
}
