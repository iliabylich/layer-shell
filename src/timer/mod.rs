use crate::{
    liburing::IoUring,
    macros::report_and_exit,
    sansio::{Satisfy, TimerFd, Wants},
    user_data::{ModuleId, UserData},
};

pub(crate) struct Timer {
    timerfd: TimerFd,
}

impl Timer {
    pub(crate) fn new() -> Box<Self> {
        Box::new(Self {
            timerfd: TimerFd::new(),
        })
    }

    fn schedule_read(&mut self) {
        match self.timerfd.wants() {
            Wants::Read { fd, buf, len } => {
                let mut sqe = IoUring::get_sqe();
                sqe.prep_read(fd, buf, len);
                sqe.set_user_data(UserData::new(ModuleId::TimerFD, Satisfy::Read));
            }

            _ => unreachable!(),
        }
    }

    pub(crate) fn init(&mut self) {
        self.schedule_read();
    }

    pub(crate) fn process(&mut self, op: u8, res: i32) -> u64 {
        let satisfy = Satisfy::from(op);

        let tick = self
            .timerfd
            .satisfy(satisfy, res)
            .unwrap_or_else(|err| report_and_exit!("{err:?}"));

        self.schedule_read();
        tick
    }
}
