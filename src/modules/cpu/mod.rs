use crate::{Event, UserData, liburing::IoUring, user_data::ModuleId};
use libc::{AT_FDCWD, O_RDONLY};
use parser::Parser;
use store::Store;

mod parser;
mod store;

#[expect(clippy::upper_case_acronyms)]
pub(crate) struct CPU {
    fd: i32,
    buf: [u8; 1_024],
    store: Store,
    healthy: bool,
}

#[repr(u8)]
#[derive(Debug)]
enum Op {
    OpenAt,
    Read,
}
const MAX_OP: u8 = Op::Read as u8;

impl From<u8> for Op {
    fn from(value: u8) -> Self {
        if value > MAX_OP {
            log::error!("unsupported op in CPUOp: {value}");
            std::process::exit(1);
        }
        unsafe { std::mem::transmute::<u8, Self>(value) }
    }
}

impl CPU {
    pub(crate) fn new() -> Box<Self> {
        Box::new(Self {
            fd: -1,
            buf: [0; 1_024],
            store: Store::new(),
            healthy: true,
        })
    }

    fn schedule_open(&self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_openat(AT_FDCWD, c"/proc/stat".as_ptr(), O_RDONLY, 0);
        sqe.set_user_data(UserData::new(ModuleId::CPU, Op::OpenAt as u8));
    }

    fn schedule_read(&mut self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_read(self.fd, self.buf.as_mut_ptr(), self.buf.len());
        sqe.set_user_data(UserData::new(ModuleId::CPU, Op::Read as u8));
    }

    pub(crate) fn init(&self) {
        self.schedule_open();
    }

    pub(crate) fn process(&mut self, op: u8, res: i32, events: &mut Vec<Event>) {
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
            Op::OpenAt => {
                if res <= 0 {
                    crash!("{op:?}: res = {res}");
                }
                self.fd = res as i32;
            }
            Op::Read => {
                if res <= 0 {
                    crash!("{op:?}: res = {res}");
                }
                let len = res as usize;
                let s = match std::str::from_utf8(&self.buf[..len]) {
                    Ok(ok) => ok,
                    Err(err) => crash!("{op:?}: {err:?}"),
                };
                let data = match Parser::parse_all(s) {
                    Ok(ok) => ok,
                    Err(err) => crash!("{op:?} {err:?}"),
                };

                let usage_per_core = self.store.update(data);
                let event = Event::CpuUsage {
                    usage_per_core: usage_per_core.into(),
                };
                events.push(event);
            }
        }
    }

    pub(crate) fn tick(&mut self) {
        if self.fd != -1 {
            self.schedule_read();
        }
    }
}
