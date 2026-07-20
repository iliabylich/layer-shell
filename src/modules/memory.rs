use crate::{
    IoEvent,
    emitter::Emitter,
    sansio::{FileReader, Satisfy, Wants},
};
use anyhow::{Context, Result};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Memory {
    reader: FileReader,
    emitter: Emitter,
}

impl Memory {
    pub(crate) fn new(emitter: Emitter) -> Result<Self> {
        Ok(Self {
            reader: FileReader::new(c"/proc/meminfo")?,
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

        let (used, total) = parse(buf)?;
        self.emitter.emit(&IoEvent::Memory { used, total });
        Ok(())
    }
}

pub(crate) fn parse(buf: &[u8]) -> Result<(f64, f64)> {
    let s = core::str::from_utf8(buf).context("decoding error")?;
    let mut lines = s.lines();

    macro_rules! parse_mem_in_gb {
        ($line:expr, $prefix:expr) => {{
            let in_bytes = $line
                .trim_ascii_end()
                .strip_prefix($prefix)
                .context(concat!("no ", $prefix, " prefix"))?
                .strip_suffix("kB")
                .context("no 'kB' suffix")?
                .trim_ascii()
                .parse::<f64>()
                .context(concat!("not an int on ", $prefix, " line"))?;
            in_bytes / 1_024.0 / 1_024.0
        }};
    }

    let mem_total = lines.next().context("no line 1")?;
    let total_gb = parse_mem_in_gb!(mem_total, "MemTotal:");

    let _skip = lines.next().context("no line 2")?;

    let mem_available = lines.next().context("no line 3")?;
    let available_gb = parse_mem_in_gb!(mem_available, "MemAvailable:");

    let used_gb = total_gb - available_gb;

    Ok((used_gb, total_gb))
}
