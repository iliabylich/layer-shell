use std::{fs::File, os::fd::IntoRawFd};

use crate::{
    Event, UserData,
    liburing::{Actor, IoUring},
    timerfd::Tick,
};
use anyhow::{Context as _, Result, ensure};

#[derive(Debug)]
enum State {
    WaitingForTimer,
    CanRead,
    Reading,
}

pub(crate) struct CPU {
    fd: i32,
    state: State,
    buf: [u8; 1_024],
    store: Option<Vec<Core>>,
}

impl CPU {
    pub(crate) fn new() -> Result<Box<Self>> {
        Ok(Box::new(Self {
            fd: File::open("/proc/stat")?.into_raw_fd(),
            state: State::WaitingForTimer,
            buf: [0; 1_024],
            store: None,
        }))
    }

    fn update_store(&mut self, next: Vec<Core>) -> Result<Event> {
        let previous = self.store.take();

        let usage_per_core = if let Some(previous) = previous {
            ensure!(
                previous.len() == next.len(),
                "number of CPU cores has changed from {} to {} (bug?)",
                previous.len(),
                next.len()
            );

            fn load_comparing_to(next: &Core, prev: &Core) -> Result<u8> {
                ensure!(
                    next.id == prev.id,
                    "CPU id mismatch: {} vs {}",
                    next.id,
                    prev.id
                );

                let idle_d = (next.idle - prev.idle) as f64;
                let total_d = (next.total - prev.total) as f64;

                Ok((100.0 * (1.0 - idle_d / total_d)) as u8)
            }

            previous
                .iter()
                .zip(next.iter())
                .map(|(prev, next)| load_comparing_to(next, prev))
                .collect::<Result<Vec<_>>>()?
        } else {
            vec![0; next.len()]
        };

        self.store = Some(next);
        Ok(Event::CpuUsage {
            usage_per_core: usage_per_core.into(),
        })
    }
}

const READ_USER_DATA: UserData = UserData::CpuRead;

impl Actor for CPU {
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
            let data = Core::parse_all(s)?;

            let event = self.update_store(data)?;
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

#[derive(Debug, Clone)]
pub(crate) struct Core {
    pub(crate) id: u8,
    pub(crate) idle: u64,
    pub(crate) total: u64,
}

impl Core {
    fn parse_line(line: &str) -> Result<Self> {
        let mut parts = line.split(" ");

        fn cut_str<'a>(
            i: &mut impl Iterator<Item = &'a str>,
            idx: usize,
            name: &str,
        ) -> Result<&'a str> {
            i.next()
                .with_context(|| format!("no {idx} item ({name}) in CPU line"))
        }

        fn cut_u64<'a>(
            i: &mut impl Iterator<Item = &'a str>,
            idx: usize,
            name: &str,
        ) -> Result<u64> {
            let s = cut_str(i, idx, name)?;
            s.parse()
                .with_context(|| format!("non-int {name} component: {s}"))
        }

        let id = cut_str(&mut parts, 0, "cpuN")?
            .strip_prefix("cpu")
            .context("no 'cpu' prefix in CPU line")?
            .parse::<u8>()
            .context("non-int CPU")?;

        let user_n = cut_u64(&mut parts, 1, "user")?;
        let nice_n = cut_u64(&mut parts, 2, "nice")?;
        let system_n = cut_u64(&mut parts, 3, "system")?;
        let idle_n = cut_u64(&mut parts, 4, "idle")?;
        let iowait_n = cut_u64(&mut parts, 5, "iowait")?;
        let irq_n = cut_u64(&mut parts, 6, "irq")?;
        let softirq_n = cut_u64(&mut parts, 7, "softirq")?;
        let steal_n = cut_u64(&mut parts, 8, "steal")?;
        let guest_n = cut_u64(&mut parts, 9, "guest")?;
        let guest_nice_n = cut_u64(&mut parts, 10, "guest_nice")?;

        let idle = idle_n + iowait_n;
        let total = user_n
            + nice_n
            + system_n
            + idle_n
            + iowait_n
            + irq_n
            + softirq_n
            + steal_n
            + guest_n
            + guest_nice_n;

        Ok(Self { id, idle, total })
    }

    pub(crate) fn parse_all(s: &str) -> Result<Vec<Self>> {
        s.lines()
            .filter(|line| line.starts_with("cpu") && line.as_bytes()[3].is_ascii_digit())
            .map(|line| {
                Self::parse_line(line).with_context(|| format!("failed to parse line '{line}'"))
            })
            .collect::<Result<Vec<_>, _>>()
    }
}
