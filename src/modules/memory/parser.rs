use anyhow::{Context as _, Result};

pub(crate) struct Parser;

impl Parser {
    pub(crate) fn parse(contents: &str) -> Result<(f64, f64)> {
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

        let mem_total = lines.next().context("no line 1")?;
        let total_kb = parse_mem(mem_total, "MemTotal:")?;

        let _skip = lines.next().context("no line 2")?;

        let mem_available = lines.next().context("no line 3")?;
        let available_kb = parse_mem(mem_available, "MemAvailable:")?;

        let used_kb = total_kb - available_kb;

        Ok((
            (f64::from(u32::try_from(used_kb).context("used KB is too long")?)) / 1024.0 / 1024.0,
            (f64::from(u32::try_from(total_kb).context("total KB is too long")?)) / 1024.0 / 1024.0,
        ))
    }
}
