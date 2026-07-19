use crate::{
    Event,
    emitter::Emitter,
    sansio::{FileReader, Satisfy, Wants},
};
use alloc::{vec, vec::Vec};
use anyhow::{Context as _, Result, bail};

pub(crate) struct Cpu {
    reader: FileReader,
    state: Option<Vec<CoreUsage>>,
    emitter: Emitter,
}

impl Cpu {
    pub(crate) fn new(emitter: Emitter) -> Result<Self> {
        Ok(Self {
            reader: FileReader::new(c"/proc/stat")?,
            state: None,
            emitter,
        })
    }

    pub(crate) const fn tick(&mut self) {
        self.reader.tick();
    }

    pub(crate) const fn wants(&mut self, buf: &mut [u8]) -> Option<Wants> {
        self.reader.wants(buf)
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, buf: &[u8]) -> Result<()> {
        let Some(buf) = self.reader.try_satisfy(satisfy, buf)? else {
            return Ok(());
        };

        let prev = self.state.take();
        let next = CoreUsage::parse_many(buf)?;

        let usage_per_core = diff(prev.as_deref(), &next)?.into();
        self.state = Some(next);
        self.emitter.emit(&Event::CpuUsage { usage_per_core });
        Ok(())
    }
}

fn diff(prev: Option<&[CoreUsage]>, next: &[CoreUsage]) -> Result<Vec<u8>> {
    let Some(prev) = prev else {
        return Ok(vec![0; next.len()]);
    };

    prev.iter()
        .zip(next.iter())
        .map(|(prev, next)| next.load_comparing_to(*prev))
        .collect()
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CoreUsage {
    pub(crate) id: u8,
    pub(crate) idle: u64,
    pub(crate) total: u64,
}

impl CoreUsage {
    pub(crate) fn parse_many(buf: &[u8]) -> Result<Vec<Self>> {
        let s = core::str::from_utf8(buf).context("decoding error")?;

        let is_cpu_line = |line: &str| -> bool {
            line.starts_with("cpu") && line.as_bytes().get(3).is_some_and(u8::is_ascii_digit)
        };

        s.lines()
            .filter(|line| is_cpu_line(line))
            .map(Self::parse)
            .collect()
    }

    fn parse(line: &str) -> Result<Self> {
        let mut parts = line.split(' ');

        macro_rules! cut_str {
            ($idx:expr, $name:expr) => {
                parts
                    .next()
                    .context(concat!("no ", $idx, " item (", $name, ") in CPU line"))
            };
        }
        macro_rules! cut_u64 {
            ($idx:expr, $name:expr) => {
                cut_str!($idx, $name)?.parse::<u64>().context(concat!(
                    "non-int ",
                    $name,
                    " component"
                ))
            };
        }

        let id = cut_str!(0, "cpuN")?
            .strip_prefix("cpu")
            .context("no 'cpu' prefix in CPU line")?
            .parse::<u8>()
            .context("non-int CPU")?;

        let user_n = cut_u64!(1, "user")?;
        let nice_n = cut_u64!(2, "nice")?;
        let system_n = cut_u64!(3, "system")?;
        let idle_n = cut_u64!(4, "idle")?;
        let iowait_n = cut_u64!(5, "iowait")?;
        let irq_n = cut_u64!(6, "irq")?;
        let softirq_n = cut_u64!(7, "softirq")?;
        let steal_n = cut_u64!(8, "steal")?;
        let guest_n = cut_u64!(9, "guest")?;
        let guest_nice_n = cut_u64!(10, "guest_nice")?;

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

    pub(crate) fn load_comparing_to(&self, prev: Self) -> Result<u8> {
        if self.id != prev.id {
            bail!("CPU id mismatch: {} vs {}", self.id, prev.id);
        }

        let idle_d =
            f64::from(u32::try_from(self.idle - prev.idle).context("values are too large")?);
        let total_d =
            f64::from(u32::try_from(self.total - prev.total).context("values are too large")?);
        let percent = 100.0 * (1.0 - idle_d / total_d);

        let percent = percent as i64;

        u8::try_from(percent).context("percent is too big for u8")
    }
}
