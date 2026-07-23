use crate::{IoEvent, emitter::Emitter, module_id::ModuleId, modules::Module};
use rustix::{
    fd::OwnedFd,
    fs::{Mode, OFlags},
};

#[derive(Debug)]
pub struct Memory {
    fd: OwnedFd,
}

impl Memory {
    pub(crate) fn new() -> Option<Self> {
        log::trace!("Creating Memory");

        let fd = match rustix::fs::open("/proc/meminfo", OFlags::RDONLY, Mode::empty()) {
            Ok(fd) => fd,
            Err(err) => {
                log::error!("failed to open /proc/meminfo: {err:?}");
                return None;
            }
        };

        Some(Self { fd })
    }
}

impl Module for Memory {
    fn read(&mut self, emitter: Emitter) -> Result<(), ()> {
        let mut buf = [0; 1_024];
        let len = rustix::io::pread(&self.fd, &mut buf, 0)
            .map_err(|err| log::error!("failed to read /proc/meminfo: {err:?}"))?;
        let Some(buf) = buf.get(..len) else {
            log::error!("read() returned more than asked: {len}");
            return Err(());
        };

        let (used, total) = parse(buf)?;
        emitter.emit(&IoEvent::Memory { used, total });
        Ok(())
    }

    fn id(&self) -> ModuleId {
        ModuleId::Memory
    }

    const MODULE_ID: ModuleId = ModuleId::Memory;
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
