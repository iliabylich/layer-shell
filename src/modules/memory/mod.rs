use crate::Event;
use anyhow::{Context as _, Result};

pub(crate) fn tick() -> Result<()> {
    let contents =
        std::fs::read_to_string("/proc/meminfo").context("failed to read /proc/meminfo")?;

    let mut lines = contents.lines();
    let line_total = lines.next().context("failed to get the 1st line")?;
    let _ = lines.next().context("failed to get the 2nd line")?;
    let line_available = lines.next().context("failed to get the 3rd line")?;

    let parse_mem = |line: &str, prefix: &str| {
        line.strip_prefix(prefix)
            .with_context(|| format!("no {prefix} prefix"))?
            .strip_suffix("kB")
            .context("no 'kB' sufix")?
            .trim()
            .parse::<usize>()
            .with_context(|| format!("not an int on {prefix} line"))
    };

    let total_kb = parse_mem(line_total, "MemTotal:")?;
    let available_kb = parse_mem(line_available, "MemAvailable:")?;
    let used_kb = total_kb - available_kb;

    let event = Event::Memory {
        used: (used_kb as f64) / 1024.0 / 1024.0,
        total: (total_kb as f64) / 1024.0 / 1024.0,
    };
    event.emit();
    Ok(())
}
