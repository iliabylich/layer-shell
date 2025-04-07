use crate::{Event, channel::EventSender0, hyprctl};
use anyhow::{Context as _, Result};
use std::io::Read;

pub(crate) struct Memory {
    tx: EventSender0,
    buf: Vec<u8>,
}

impl Memory {
    pub(crate) const INTERVAL: u64 = 1;

    pub(crate) fn new(tx: EventSender0) -> Self {
        Self {
            tx,
            buf: vec![0; 1_000],
        }
    }

    pub(crate) fn tick(&mut self) {
        if let Err(err) = self.try_tick() {
            log::error!("{:?}", err);
        }
    }

    fn try_tick(&mut self) -> Result<()> {
        let mut file = std::fs::File::open("/proc/meminfo").context("failed to open")?;
        let len = file.read(&mut self.buf).context("failed to read")?;
        let contents = std::str::from_utf8(&self.buf[..len]).context("non-utf8 content")?;
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

        let event = Event::Memory {
            used: (used_kb as f64) / 1024.0 / 1024.0,
            total: (total_kb as f64) / 1024.0 / 1024.0,
        };
        self.tx.send(event);
        Ok(())
    }

    pub(crate) fn spawn_system_monitor() {
        if let Err(err) = hyprctl::dispatch("exec gnome-system-monitor") {
            log::error!("Failed to open system monitor: {:?}", err);
        }
    }
}
