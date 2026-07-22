use crate::{
    error::IoError,
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
impl State {
    const fn as_str(self) -> &'static str {
        match self {
            Self::CanRead => "CanRead",
            Self::WaitingForRead => "WaitingForRead",
        }
    }
}

impl TimerFd {
    pub(crate) fn new() -> Result<Self, IoError> {
        Ok(Self {
            fd: create_timer()?,
            ticks: 0,
            state: State::CanRead,
        })
    }

    pub(crate) const fn wants(&mut self, buf: &mut [u8; 8]) -> Option<Wants> {
        match self.state {
            State::CanRead => {
                self.state = State::WaitingForRead;

                Some(Wants::Read {
                    fd: self.fd,
                    buf: buf.as_mut_ptr(),
                    len: buf.len(),
                })
            }
            State::WaitingForRead => None,
        }
    }

    pub(crate) fn try_satisfy(
        &mut self,
        satisfy: Satisfy,
        buf: [u8; 8],
    ) -> Result<Option<u64>, IoError> {
        match (self.state, satisfy) {
            (State::WaitingForRead, Satisfy::Read(res)) => {
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
            }

            _ => Err(IoError::WrongSatisfy {
                satisfy: satisfy.as_str(),
                state: self.state.as_str(),
            }),
        }
    }
}

fn create_timer() -> Result<BorrowedFd<'static>, IoError> {
    let fd = timerfd_create(TimerfdClockId::Monotonic, TimerfdFlags::empty()).map_err(|errno| {
        IoError::FailedTo {
            op: "timerfd_create",
            errno,
        }
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
    .map_err(|errno| IoError::FailedTo {
        op: "timerfd_settime",
        errno,
    })?;

    Ok(fd)
}
