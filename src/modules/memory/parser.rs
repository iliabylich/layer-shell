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

        let line1 = lines.next().context("no line 1")?;
        let total_kb = parse_mem(line1, "MemTotal:")?;

        let _line2 = lines.next().context("no line 2")?;

        let line3 = lines.next().context("no line 3")?;
        let available_kb = parse_mem(line3, "MemAvailable:")?;

        let used_kb = total_kb - available_kb;

        Ok((
            (used_kb as f64) / 1024.0 / 1024.0,
            (total_kb as f64) / 1024.0 / 1024.0,
        ))
    }
}
