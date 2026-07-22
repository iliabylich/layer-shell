use crate::{
    sansio::{Satisfy, Wants},
    utils::log_err_and_exit,
};
use rustix::{
    fd::{BorrowedFd, IntoRawFd},
    fs::Timespec,
    time::{
        Itimerspec, TimerfdClockId, TimerfdFlags, TimerfdTimerFlags, timerfd_create,
        timerfd_settime,
    },
};

#[derive(Debug, Clone, Copy)]
pub struct TimerFd {
    fd: BorrowedFd<'static>,
    ticks: u64,
    state: State,
}

#[derive(Debug, Clone, Copy)]
enum State {
    CanRead,
    WaitingForRead,
}

impl TimerFd {
    pub(crate) fn new() -> Option<Self> {
        log::trace!("Creating TimerFd");

        match create_timer() {
            Ok(fd) => Some(Self {
                fd,
                ticks: 0,
                state: State::CanRead,
            }),
            Err(()) => None,
        }
    }

    pub(crate) fn wants(&mut self, buf: &mut [u8; 8]) -> Option<Wants> {
        let wants = match self.state {
            State::CanRead => {
                self.state = State::WaitingForRead;

                Wants::Read {
                    fd: self.fd,
                    buf: buf.as_mut_ptr(),
                    len: buf.len(),
                }
            }
            State::WaitingForRead => return None,
        };
        log::trace!("{wants:?}");
        Some(wants)
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, buf: [u8; 8]) -> Result<Option<u64>, ()> {
        if let (State::WaitingForRead, Satisfy::Read(res)) = (self.state, satisfy) {
            let bytes_read = res?;
            if bytes_read != buf.len() {
                log_err_and_exit!(
                    "Timerfd: buffer is too short: {bytes_read} vs {}",
                    buf.len()
                );
            }
            let expirations = u64::from_ne_bytes(buf);

            let ticks = self.ticks;
            self.ticks = self.ticks.saturating_add(expirations);
            self.state = State::CanRead;

            Ok(Some(ticks))
        } else {
            log::error!("wrong satisfy {satisfy:?} for {self:?}");
            Err(())
        }
    }
}

fn create_timer() -> Result<BorrowedFd<'static>, ()> {
    let fd = timerfd_create(TimerfdClockId::Monotonic, TimerfdFlags::empty()).map_err(|errno| {
        log::error!("failed to timerfd_create: {errno:?}");
    })?;
    let fd = unsafe { BorrowedFd::borrow_raw(fd.into_raw_fd()) };

    timerfd_settime(
        fd,
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
    .map_err(|errno| {
        log::error!("failed to timerfd_settime: {errno:?}");
    })?;

    Ok(fd)
}
