use crate::utils::singleton;
use anyhow::{Context, Result};
use tokio::{fs::File, io::AsyncReadExt};

pub(crate) struct Memory {
    callbacks: Vec<fn(MemoryData)>,
}
singleton!(Memory);

impl Memory {
    pub(crate) async fn spawn() {
        Self::set(Self { callbacks: vec![] });

        loop {
            match parse().await {
                Ok(data) => Self::changed(data),
                Err(err) => {
                    eprintln!("failed to read system memory:\n{}", err)
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }

    pub(crate) fn subscribe(f: fn(MemoryData)) {
        this().callbacks.push(f);
    }

    fn changed(data: MemoryData) {
        for callback in this().callbacks.iter() {
            (callback)(data)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct MemoryData {
    pub(crate) used: f64,
    pub(crate) total: f64,
}

async fn parse() -> Result<MemoryData> {
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

    Ok(MemoryData {
        used: (used_kb as f64) / 1024.0 / 1024.0,
        total: (total_kb as f64) / 1024.0 / 1024.0,
    })
}
