use crate::{
    FixedSizeArrray, IoEvent,
    emitter::Emitter,
    sansio::{FileReader, Satisfy, Wants},
    utils::log_err_and_exit,
};

pub struct Cpu {
    reader: FileReader,
    state: Option<FixedSizeArrray<MAX_CPU_COUNT, CoreUsage>>,
    emitter: Emitter,
}

pub const MAX_CPU_COUNT: usize = 32;

impl Cpu {
    pub(crate) fn new(emitter: Emitter) -> Option<Self> {
        log::trace!("Creating Cpu");

        match FileReader::new(c"/proc/stat") {
            Ok(reader) => Some(Self {
                reader,
                state: None,
                emitter,
            }),
            Err(err) => {
                log::error!("{err:?}");
                None
            }
        }
    }

    pub(crate) const fn tick(&mut self) {
        self.reader.tick();
    }

    pub(crate) fn wants(&mut self, buf: &mut [u8]) -> Option<Wants> {
        let wants = self.reader.wants(buf)?;
        log::trace!("{wants:?}");
        Some(wants)
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, buf: &[u8]) -> Result<(), ()> {
        let Some(buf) = self.reader.try_satisfy(satisfy, buf)? else {
            return Ok(());
        };

        let prev = self.state.take();
        let next = CoreUsage::parse_many(buf)?;

        let usage_per_core = diff(prev.as_ref(), &next)?;
        self.state = Some(next);
        self.emitter.emit(&IoEvent::CpuUsage { usage_per_core });
        Ok(())
    }
}

fn diff(
    prev: Option<&FixedSizeArrray<MAX_CPU_COUNT, CoreUsage>>,
    next: &FixedSizeArrray<MAX_CPU_COUNT, CoreUsage>,
) -> Result<FixedSizeArrray<MAX_CPU_COUNT, u8>, ()> {
    let Some(prev) = prev else {
        return Ok(FixedSizeArrray::filled(0, next.len()));
    };

    if prev.len() != next.len() {
        log_err_and_exit!("dynamic number of CPU cores");
    }
    let len = next.len();

    let mut out = FixedSizeArrray::new();

    for idx in 0..len {
        let prev_per_core = prev.get(idx).unwrap_or_else(|| log_err_and_exit!("bug"));
        let next_per_core = next.get(idx).unwrap_or_else(|| log_err_and_exit!("bug"));
        let diff = next_per_core.load_comparing_to(*prev_per_core)?;
        out.push(diff).unwrap_or_else(|| log_err_and_exit!("bug"));
    }

    Ok(out)
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CoreUsage {
    pub(crate) id: u8,
    pub(crate) idle: u64,
    pub(crate) total: u64,
}

impl CoreUsage {
    pub(crate) fn parse_many(buf: &[u8]) -> Result<FixedSizeArrray<MAX_CPU_COUNT, Self>, ()> {
        let s = core::str::from_utf8(buf).map_err(|err| {
            log::error!("non-utf8 input: {err:?}");
        })?;

        let mut out = FixedSizeArrray::new();

        for line in s.lines() {
            if let Some(line) = line.strip_prefix("cpu")
                && line.as_bytes().first().is_some_and(u8::is_ascii_digit)
            {
                out.push(Self::parse(line)?).ok_or_else(|| {
                    log::error!("too many CPU cores");
                })?;
            }
        }

        Ok(out)
    }

    fn parse(line: &str) -> Result<Self, ()> {
        let mut parts = line.split(' ');

        let id = cut_str(&mut parts, "cpuN")?.parse::<u8>().map_err(|err| {
            log::error!("non integer field CPU id: {err:?}");
        })?;

        let user_n = cut_u64(&mut parts, "user")?;
        let nice_n = cut_u64(&mut parts, "nice")?;
        let system_n = cut_u64(&mut parts, "system")?;
        let idle_n = cut_u64(&mut parts, "idle")?;
        let iowait_n = cut_u64(&mut parts, "iowait")?;
        let irq_n = cut_u64(&mut parts, "irq")?;
        let softirq_n = cut_u64(&mut parts, "softirq")?;
        let steal_n = cut_u64(&mut parts, "steal")?;
        let guest_n = cut_u64(&mut parts, "guest")?;
        let guest_nice_n = cut_u64(&mut parts, "guestnice")?;

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

    pub(crate) fn load_comparing_to(&self, prev: Self) -> Result<u8, ()> {
        if self.id != prev.id {
            log::error!("CPU ids mismatch: {} vs {}", self.id, prev.id);
            return Err(());
        }

        let idle_d = f64::from(u32::try_from(self.idle - prev.idle).map_err(|err| {
            log::error!("failed to calculate idle_d: {err:?}");
        })?);
        let total_d = f64::from(u32::try_from(self.total - prev.total).map_err(|err| {
            log::error!("failed to calculate total_d: {err:?}");
        })?);
        let percent = 100.0 * (1.0 - idle_d / total_d);

        #[expect(clippy::cast_possible_truncation)]
        let percent = percent as i64;

        u8::try_from(percent).map_err(|err| {
            log::error!("failed to calculate load_comparing_to final percentage: {err:?}");
        })
    }
}

fn cut_str<'a>(parts: &mut impl Iterator<Item = &'a str>, field: &str) -> Result<&'a str, ()> {
    parts.next().ok_or_else(|| {
        log::error!("no field {field:?}");
    })
}

fn cut_u64<'a>(parts: &mut impl Iterator<Item = &'a str>, field: &str) -> Result<u64, ()> {
    let s = cut_str(parts, field)?;
    s.parse::<u64>().map_err(|err| {
        log::error!("non integer field {field:?}: {err:?}");
    })
}
