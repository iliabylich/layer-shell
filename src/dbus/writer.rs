use crate::{
    dbus::ConnectionKind,
    liburing::IoUring,
    macros::define_op,
    user_data::{ModuleId, UserData},
};
use anyhow::{Result, ensure};

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

    fn try_process(&self, op: Op, res: i32) -> Result<()> {
        match op {
            Op::Write => {
                ensure!(res >= 0);
                let written = res as usize;
                ensure!(
                    written == self.buf.len(),
                    "written is wrong: {written} vs {}",
                    self.buf.len()
                );
                Ok(())
            }
        }
    }

    pub(crate) fn process(&mut self, op: u8, res: i32) {
        if !self.healthy {
            return;
        }

        let op = Op::from(op);

        if let Err(err) = self.try_process(op, res) {
            log::error!("DBusReader({:?})::{op:?}({res} {err:?}", self.kind);
            self.healthy = false;
        }
    }
}
