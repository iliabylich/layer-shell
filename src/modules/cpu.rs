use crate::{
    FixedSizeArrray, IoEvent,
    emitter::Emitter,
    error::IoError,
    sansio::{FileReader, Satisfy, Wants},
};
use thiserror::Error;

pub(crate) struct Cpu {
    reader: FileReader,
    state: Option<FixedSizeArrray<MAX_CPU_COUNT, CoreUsage>>,
    emitter: Emitter,
}

pub const MAX_CPU_COUNT: usize = 32;

impl Cpu {
    pub(crate) fn new(emitter: Emitter) -> Result<Self, IoError> {
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

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, buf: &[u8]) -> Result<(), IoError> {
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
) -> Result<FixedSizeArrray<MAX_CPU_COUNT, u8>, CpuError> {
    let Some(prev) = prev else {
        return Ok(FixedSizeArrray::filled(0, next.len()));
    };

    debug_assert_eq!(prev.len(), next.len());
    let len = next.len();

    let mut out = FixedSizeArrray::new();

    for idx in 0..len {
        let prev_per_core = prev.get(idx).unwrap_or_else(|| unreachable!());
        let next_per_core = next.get(idx).unwrap_or_else(|| unreachable!());
        let diff = next_per_core.load_comparing_to(*prev_per_core)?;
        out.push(diff).unwrap_or_else(|| unreachable!());
    }

    Ok(out)
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct CoreUsage {
    pub(crate) id: u8,
    pub(crate) idle: u64,
    pub(crate) total: u64,
}

impl CoreUsage {
    pub(crate) fn parse_many(buf: &[u8]) -> Result<FixedSizeArrray<MAX_CPU_COUNT, Self>, CpuError> {
        let s = core::str::from_utf8(buf).map_err(CpuError::MalformedInput)?;

        let mut out = FixedSizeArrray::new();

        for line in s.lines() {
            if let Some(line) = line.strip_prefix("cpu")
                && line.as_bytes().first().is_some_and(u8::is_ascii_digit)
            {
                out.push(Self::parse(line)?)
                    .ok_or(CpuError::TooManyCpuCores)?;
            }
        }

        Ok(out)
    }

    fn parse(line: &str) -> Result<Self, CpuError> {
        let mut parts = line.split(' ');

        macro_rules! cut_str {
            ($field:expr) => {
                parts.next().ok_or(CpuError::NoField { field: $field })
            };
        }
        macro_rules! cut_u64 {
            ($field:expr) => {
                cut_str!($field)?
                    .parse::<u64>()
                    .map_err(|err| CpuError::NonIntegerField {
                        field: $field,
                        kind: *err.kind(),
                    })
            };
        }

        let id = cut_str!("cpuN")?
            .parse::<u8>()
            .map_err(|err| CpuError::NonIntegerField {
                field: "id",
                kind: *err.kind(),
            })?;

        let user_n = cut_u64!("user")?;
        let nice_n = cut_u64!("nice")?;
        let system_n = cut_u64!("system")?;
        let idle_n = cut_u64!("idle")?;
        let iowait_n = cut_u64!("iowait")?;
        let irq_n = cut_u64!("irq")?;
        let softirq_n = cut_u64!("softirq")?;
        let steal_n = cut_u64!("steal")?;
        let guest_n = cut_u64!("guest")?;
        let guest_nice_n = cut_u64!("guestnice")?;

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

    pub(crate) fn load_comparing_to(&self, prev: Self) -> Result<u8, CpuError> {
        if self.id != prev.id {
            return Err(CpuError::CpuIdMismatch {
                next: self.id,
                prev: prev.id,
            });
        }

        let idle_d = f64::from(
            u32::try_from(self.idle - prev.idle).map_err(|_| CpuError::IdleDiffIsTooBig)?,
        );
        let total_d = f64::from(
            u32::try_from(self.total - prev.total).map_err(|_| CpuError::TotalDiffIsTooBig)?,
        );
        let percent = 100.0 * (1.0 - idle_d / total_d);

        let percent = percent as i64;

        u8::try_from(percent).map_err(|_| CpuError::PercentageIsTooBig)
    }
}

#[derive(Debug, Error, Clone, Copy)]
pub(crate) enum CpuError {
    #[error("non-utf8 CPU data")]
    MalformedInput(core::str::Utf8Error),
    #[error("too many CPU cores")]
    TooManyCpuCores,
    #[error("missing CPU field {field:?}")]
    NoField { field: &'static str },
    #[error("failed to parse CPU field {field:?}")]
    NonIntegerField {
        field: &'static str,
        kind: core::num::IntErrorKind,
    },
    #[error("CPU id mismatch: next {next}, prev {prev}")]
    CpuIdMismatch { next: u8, prev: u8 },
    #[error("CPU idle diff is too big")]
    IdleDiffIsTooBig,
    #[error("CPU total diff is too big")]
    TotalDiffIsTooBig,
    #[error("CPU percent is too big")]
    PercentageIsTooBig,
}
