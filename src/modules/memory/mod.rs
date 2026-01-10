use crate::{Event, UserData, liburing::IoUring, timerfd::Tick};
use anyhow::{Result, ensure};
use parser::Parser;
use std::{fs::File, os::fd::IntoRawFd};

mod parser;

#[derive(Debug)]
enum State {
    WaitingForTimer,
    CanRead,
    Reading,
}

pub(crate) struct Memory {
    fd: i32,
    state: State,
    buf: [u8; 1_024],
}

const READ_USER_DATA: UserData = UserData::MemoryRead;

impl Memory {
    pub(crate) fn new() -> Result<Box<Self>> {
        Ok(Box::new(Self {
            fd: File::open("/proc/meminfo")?.into_raw_fd(),
            state: State::WaitingForTimer,
            buf: [0; 1_024],
        }))
    }

    pub(crate) fn drain(&mut self, ring: &mut IoUring) -> Result<bool> {
        match self.state {
            State::CanRead => {
                let mut sqe = ring.get_sqe()?;
                sqe.prep_read(self.fd, self.buf.as_mut_ptr(), self.buf.len());
                sqe.set_user_data(READ_USER_DATA.as_u64());

                self.state = State::Reading;
                Ok(true)
            }
            State::Reading => Ok(false),

            State::WaitingForTimer => Ok(false),
        }
    }

    pub(crate) fn feed(
        &mut self,
        user_data: UserData,
        res: i32,
        events: &mut Vec<Event>,
    ) -> Result<()> {
        if user_data == READ_USER_DATA {
            ensure!(
                matches!(self.state, State::Reading),
                "malformed state, expected Reading, got {:?}",
                self.state
            );

            ensure!(res > 0);
            let len = res as usize;
            let s = std::str::from_utf8(&self.buf[..len])?;

            let (used, total) = Parser::parse(s)?;
            events.push(Event::Memory { used, total });

            self.state = State::WaitingForTimer;
            return Ok(());
        }

        Ok(())
    }

    pub(crate) fn on_tick(&mut self, tick: Tick) -> Result<()> {
        if tick.is_multiple_of(1) {
            assert!(
                matches!(self.state, State::WaitingForTimer),
                "malformed state, expected WaitingForTimer, got {:?}",
                self.state,
            );
            self.state = State::CanRead;
        }
        Ok(())
    }
}
