use crate::{Event, UserData, liburing::IoUring, macros::define_op, user_data::ModuleId};
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

define_op!("CPU", OpenAt, Read);

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
        sqe.set_user_data(UserData::new(ModuleId::CPU, Op::OpenAt));
    }

    fn schedule_read(&mut self) {
        let mut sqe = IoUring::get_sqe();
        sqe.prep_read(self.fd, self.buf.as_mut_ptr(), self.buf.len());
        sqe.set_user_data(UserData::new(ModuleId::CPU, Op::Read));
    }

    pub(crate) fn init(&self) {
        self.schedule_open();
    }

    pub(crate) fn process(&mut self, op: u8, res: i32, events: &mut Vec<Event>) {
        if !self.healthy {
            return;
        }

        let op = Op::from(op);

        macro_rules! assert_or_unhealthy {
            ($cond:expr, $($arg:tt)*) => {
                if !$cond {
                    log::error!("CPU::{op:?}");
                    log::error!($($arg)*);
                    self.healthy = false;
                    return;
                }
            };
        }

        match op {
            Op::OpenAt => {
                assert_or_unhealthy!(res > 0, "res is {res}");
                self.fd = res;
            }
            Op::Read => {
                assert_or_unhealthy!(res >= 0, "res is {res}");
                let len = res as usize;

                let s = std::str::from_utf8(&self.buf[..len]);
                assert_or_unhealthy!(s.is_ok(), "decoding error: {s:?}");
                let s = unsafe { s.unwrap_unchecked() };

                let data = Parser::parse_all(s);
                assert_or_unhealthy!(data.is_ok(), "parse error: {data:?}");
                let data = unsafe { data.unwrap_unchecked() };

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
