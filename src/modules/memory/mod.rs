use crate::{Event, UserData, liburing::IoUring, user_data::ModuleId};
use libc::{AT_FDCWD, O_RDONLY};
use parser::Parser;

mod parser;

pub(crate) struct Memory {
    fd: i32,
    buf: [u8; 1_024],
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
            log::error!("unsupported op in MemoryOp: {value}");
            std::process::exit(1);
        }
        unsafe { std::mem::transmute::<u8, Self>(value) }
    }
}

impl Memory {
    pub(crate) fn new() -> Box<Self> {
        Box::new(Self {
            fd: -1,
            buf: [0; 1_024],
            healthy: true,
        })
    }

    fn schedule_open(&self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_openat(AT_FDCWD, c"/proc/meminfo".as_ptr(), O_RDONLY, 0);
        sqe.set_user_data(UserData::new(ModuleId::Memory, Op::OpenAt as u8));
    }

    fn schedule_read(&mut self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_read(self.fd, self.buf.as_mut_ptr(), self.buf.len());
        sqe.set_user_data(UserData::new(ModuleId::Memory, Op::Read as u8));
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
                    Err(err) => crash!("{op:?} {err:?}"),
                };

                let (used, total) = match Parser::parse(s) {
                    Ok(ok) => ok,
                    Err(err) => crash!("{op:?} {err:?}"),
                };
                events.push(Event::Memory { used, total });
            }
        }
    }

    pub(crate) fn tick(&mut self) {
        if self.fd != -1 {
            self.schedule_read();
        }
    }
}
