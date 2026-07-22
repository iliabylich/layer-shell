use thiserror::Error;

use crate::{
    IoEvent,
    emitter::Emitter,
    error::IoError,
    sansio::{FileReader, Satisfy, Wants},
};

#[derive(Debug, Clone, Copy)]
pub struct Memory {
    reader: FileReader,
    emitter: Emitter,
}

impl Memory {
    pub(crate) fn new(emitter: Emitter) -> Option<Self> {
        match FileReader::new(c"/proc/meminfo") {
            Ok(reader) => Some(Self { reader, emitter }),
            Err(err) => {
                log::error!(target: "Memory", "{err:?}");
                None
            }
        }
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

        let (used, total) = parse(buf)?;
        self.emitter.emit(&IoEvent::Memory { used, total });
        Ok(())
    }
}

fn parse(buf: &[u8]) -> Result<(f64, f64), MemoryError> {
    let s = core::str::from_utf8(buf).map_err(MemoryError::Decode)?;
    let mut lines = s.lines();

    macro_rules! parse_mem_in_gb {
        ($line:expr, $prefix:expr) => {{
            let in_bytes = $line
                .trim_ascii_end()
                .strip_prefix($prefix)
                .ok_or(MemoryError::MissingPrefix { prefix: $prefix })?
                .strip_suffix("kB")
                .ok_or(MemoryError::MissingKbSuffix { prefix: $prefix })?
                .trim_ascii()
                .parse::<f64>()
                .map_err(|_| MemoryError::ParseFloat { prefix: $prefix })?;
            in_bytes / 1_024.0 / 1_024.0
        }};
    }

    let mem_total = lines.next().ok_or(MemoryError::MissingLine { line: 1 })?;
    let total_gb = parse_mem_in_gb!(mem_total, "MemTotal:");

    let _skip = lines.next().ok_or(MemoryError::MissingLine { line: 2 })?;

    let mem_available = lines.next().ok_or(MemoryError::MissingLine { line: 3 })?;
    let available_gb = parse_mem_in_gb!(mem_available, "MemAvailable:");

    let used_gb = total_gb - available_gb;

    Ok((used_gb, total_gb))
}

#[derive(Debug, Error, Clone, Copy)]
pub enum MemoryError {
    #[error("non-utf8 memory data")]
    Decode(core::str::Utf8Error),
    #[error("missing memory line {line}")]
    MissingLine { line: usize },
    #[error("missing memory prefix {prefix}")]
    MissingPrefix { prefix: &'static str },
    #[error("missing kB suffix for {prefix}")]
    MissingKbSuffix { prefix: &'static str },
    #[error("failed to parse memory value for {prefix}")]
    ParseFloat { prefix: &'static str },
}
