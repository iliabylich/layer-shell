use anyhow::{Context as _, Result};
use async_stream::stream;
use futures::Stream;
use tokio::{fs::File, io::AsyncReadExt as _};

#[derive(Debug)]
pub struct Memory {
    pub used: f64,
    pub total: f64,
}

pub fn connect() -> impl Stream<Item = Memory> {
    stream! {
        loop {
            match Memory::parse().await {
                Ok(mem) => yield mem,
                Err(err) => log::error!("failed to get memory info: {:?}", err),
            }
        }
    }
}

impl Memory {
    async fn parse() -> Result<Self> {
        let mut f = File::open("/proc/meminfo")
            .await
            .context("failed to open /proc/meminfo")?;
        let mut contents = String::new();
        f.read_to_string(&mut contents)
            .await
            .context("failed to read /proc/meminfo")?;

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

        Ok(Self {
            used: (used_kb as f64) / 1024.0 / 1024.0,
            total: (total_kb as f64) / 1024.0 / 1024.0,
        })
    }
}
