use crate::{
    IoEvent,
    emitter::Emitter,
    sansio::{FileReader, Satisfy, Wants},
};

#[derive(Debug, Clone, Copy)]
pub struct Memory {
    reader: FileReader,
    emitter: Emitter,
}

impl Memory {
    pub(crate) fn new(emitter: Emitter) -> Option<Self> {
        log::trace!("Creating Memory");

        match FileReader::new(c"/proc/meminfo") {
            Ok(reader) => Some(Self { reader, emitter }),
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

        let (used, total) = parse(buf)?;
        self.emitter.emit(&IoEvent::Memory { used, total });
        Ok(())
    }
}

fn parse(buf: &[u8]) -> Result<(f64, f64), ()> {
    let text = core::str::from_utf8(buf).map_err(|err| {
        log::error!("non-utf8 input: {err:?}");
    })?;
    let mut lines = text.lines();

    let mem_total = nextline(&mut lines, text)?;
    let _ = nextline(&mut lines, text)?;
    let mem_available = nextline(&mut lines, text)?;

    let mem_total = parse_mem_in_gb(mem_total, "MemTotal:")?;
    let mem_available = parse_mem_in_gb(mem_available, "MemAvailable:")?;
    let mem_used = mem_total - mem_available;

    Ok((mem_used, mem_total))
}

fn parse_mem_in_gb(line: &str, prefix: &'static str) -> Result<f64, ()> {
    let mem_in_kb = line
        .trim_ascii_end()
        .strip_prefix(prefix)
        .ok_or_else(|| {
            log::error!("missing prefix {prefix:?} in line {line:?}");
        })?
        .strip_suffix("kB")
        .ok_or_else(|| {
            log::error!("missing kB suffix in line {line:?}");
        })?
        .trim_ascii()
        .parse::<f64>()
        .map_err(|err| {
            log::error!("failed to parse value in line {line:?} as float: {err:?}");
        })?;
    Ok(mem_in_kb / 1_024.0 / 1_024.0)
}

fn nextline<'a>(lines: &mut impl Iterator<Item = &'a str>, text: &str) -> Result<&'a str, ()> {
    lines.next().ok_or_else(|| {
        log::error!("no line 3 in {text:?}");
    })
}
