use crate::models::Event;
use anyhow::{Context, Result};
use tokio::{fs::File, io::AsyncReadExt, sync::mpsc::Sender};

pub(crate) async fn spawn(tx: Sender<Event>) {
    if let Err(err) = try_spawn(tx).await {
        eprintln!("Memory model error:\n{}\n{}", err, err.backtrace());
        return;
    }
}

async fn try_spawn(tx: Sender<Event>) -> Result<()> {
    loop {
        let data = parse().await.context("failed to get memory info")?;
        tx.send(Event::Memory {
            used: data.used,
            total: data.total,
        })
        .await
        .context("failed to send event")?;

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

#[derive(Debug, Clone, Copy)]
struct MemoryData {
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
