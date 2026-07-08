use anyhow::{Context as _, Result};

pub(crate) struct Parser;

impl Parser {
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
}
