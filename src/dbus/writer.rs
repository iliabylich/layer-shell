use crate::{
    dbus::ConnectionKind,
    liburing::IoUring,
    macros::report_and_exit,
    user_data::{ModuleId, UserData},
};

#[repr(u8)]
#[derive(Debug)]
enum Op {
    Write,
}
const MAX_OP: u8 = Op::Write as u8;

impl From<u8> for Op {
    fn from(value: u8) -> Self {
        if value > MAX_OP {
            report_and_exit!("unsupported op in DBus Writer: {value}");
        }
        unsafe { std::mem::transmute::<u8, Self>(value) }
    }
}

pub(crate) struct Writer {
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
            fd: -1,
            module_id,
            buf: vec![],
            healthy: true,
        }
    }

    fn schedule_write(&mut self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_write(self.fd, self.buf.as_ptr(), self.buf.len());
        sqe.set_user_data(UserData::new(self.module_id, Op::Write as u8));
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

        macro_rules! crash {
            ($($arg:tt)*) => {{
                log::error!($($arg)*);
                self.healthy = false;
                return;
            }};
        }

        match op {
            Op::Write => {
                if res <= 0 {
                    crash!("{op:?}: res is {res}");
                }
                let written = res as usize;
                if written != self.buf.len() {
                    crash!("{op:?}: written is wrong: {written} vs {}", self.buf.len());
                }
            }
        }
    }
}
