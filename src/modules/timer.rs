use rustix::{
    fd::{AsFd, BorrowedFd, OwnedFd},
    fs::Timespec,
    time::{
        Itimerspec, TimerfdClockId, TimerfdFlags, TimerfdTimerFlags, timerfd_create,
        timerfd_settime,
    },
};

use crate::{emitter::Emitter, module_id::ModuleId, modules::Module};

#[derive(Debug)]
pub struct Timer {
    fd: OwnedFd,
}

impl Timer {
    pub(crate) fn new() -> Option<Self> {
        log::trace!("Creating Timer");

        match create_timer() {
            Ok(fd) => Some(Self { fd }),
            Err(()) => None,
        }
    }
}

impl Module for Timer {
    fn read(&mut self, _emitter: Emitter) -> Result<(), ()> {
        let mut buf = [0; 8];
        let len = rustix::io::read(&self.fd, &mut buf).map_err(|err| {
            log::error!("failed to read: {err:?}");
        })?;
        assert_eq!(len, 8);

        Ok(())
    }

    fn id(&self) -> ModuleId {
        ModuleId::Timer
    }

    const MODULE_ID: ModuleId = ModuleId::Timer;
}

impl AsFd for Timer {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

fn create_timer() -> Result<OwnedFd, ()> {
    let fd = timerfd_create(TimerfdClockId::Monotonic, TimerfdFlags::empty()).map_err(|err| {
        log::error!("failed to timerfd_create: {err:?}");
    })?;

    timerfd_settime(
        &fd,
        TimerfdTimerFlags::empty(),
        &Itimerspec {
            it_interval: Timespec {
                tv_sec: 1,
                tv_nsec: 0,
            },
            it_value: Timespec {
                tv_sec: 0,
                tv_nsec: 1,
            },
        },
    )
    .map_err(|err| {
        log::error!("failed to timerfd_settime: {err:?}");
    })?;

    Ok(fd)
}
