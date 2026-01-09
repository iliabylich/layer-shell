use std::{fs::File, os::fd::IntoRawFd};

use crate::{
    Event, UserData,
    liburing::{IoUring, IoUringActor},
    timerfd::Tick,
};
use anyhow::{Context as _, Result, ensure};

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

impl Memory {
    pub(crate) fn new() -> Result<Box<Self>> {
        Ok(Box::new(Self {
            fd: File::open("/proc/meminfo")?.into_raw_fd(),
            state: State::WaitingForTimer,
            buf: [0; 1_024],
        }))
    }
}

const READ_USER_DATA: UserData = UserData::MemoryRead;

impl IoUringActor for Memory {
    fn drain_once(&mut self, ring: &mut IoUring, _events: &mut Vec<Event>) -> Result<bool> {
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

    fn feed(
        &mut self,
        _ring: &mut IoUring,
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

            let event = parse(s)?;
            events.push(event);

            self.state = State::WaitingForTimer;
            return Ok(());
        }

        Ok(())
    }

    fn on_tick(&mut self, tick: Tick) -> Result<()> {
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

fn parse(contents: &str) -> Result<Event> {
    let mut lines = contents.lines();

    let parse_mem = |line: &str, prefix: &str| {
        line.trim_ascii_end()
            .strip_prefix(prefix)
            .with_context(|| format!("no {prefix} prefix"))?
            .strip_suffix("kB")
            .context("no 'kB' suffix")?
            .trim_ascii()
            .parse::<usize>()
            .with_context(|| format!("not an int on {prefix} line"))
    };

    let line1 = lines.next().context("no line 1")?;
    let total_kb = parse_mem(line1, "MemTotal:")?;

    let _line2 = lines.next().context("no line 2")?;

    let line3 = lines.next().context("no line 3")?;
    let available_kb = parse_mem(line3, "MemAvailable:")?;

    let used_kb = total_kb - available_kb;

    Ok(Event::Memory {
        used: (used_kb as f64) / 1024.0 / 1024.0,
        total: (total_kb as f64) / 1024.0 / 1024.0,
    })
}
