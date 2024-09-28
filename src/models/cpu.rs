use crate::models::Event;
use anyhow::{Context, Result};
use tokio::{fs::File, io::AsyncReadExt, sync::mpsc::Sender};

pub(crate) async fn spawn(tx: Sender<Event>) {
    if let Err(err) = try_spawn(tx).await {
        log::error!("CPU model error: {}\n{}", err, err.backtrace());
    }
}

async fn try_spawn(tx: Sender<Event>) -> Result<()> {
    let mut previous: Option<Vec<CpuCoreInfo>> = None;

    loop {
        let usage_per_core = parse(&mut previous)
            .await
            .context("failed to get CPU data")?;
        tx.send(Event::Cpu { usage_per_core })
            .await
            .context("failed to send event")?;

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

async fn parse(previous: &mut Option<Vec<CpuCoreInfo>>) -> Result<Vec<usize>> {
    let current = parse_current().await?;
    let count = current.len();

    if let Some(previous_owned) = previous.take() {
        assert_eq!(previous_owned.len(), current.len());

        let usage = previous_owned
            .iter()
            .zip(current.iter())
            .map(|(prev, next)| next.load_comparing_to(prev))
            .collect::<Vec<_>>();

        *previous = Some(current);

        Ok(usage)
    } else {
        *previous = Some(current);
        Ok(vec![0; count])
    }
}

async fn parse_current() -> Result<Vec<CpuCoreInfo>> {
    let mut f = File::open("/proc/stat")
        .await
        .context("failed to open /proc/stat")?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .await
        .context("failed to read /proc/stat")?;

    contents
        .split("\n")
        .filter(|line| line.starts_with("cpu") && line.as_bytes()[3].is_ascii_digit())
        .map(|line| {
            CpuCoreInfo::try_from(line).with_context(|| format!("failed to parse line '{line}'"))
        })
        .collect::<Result<Vec<_>, _>>()
}

pub(crate) struct CpuCoreInfo {
    id: usize,
    idle: usize,
    total: usize,
}

impl CpuCoreInfo {
    fn load_comparing_to(&self, previous: &Self) -> usize {
        assert_eq!(self.id, previous.id);

        let idle_d = (self.idle - previous.idle) as f64;
        let total_d = (self.total - previous.total) as f64;

        (100.0 * (1.0 - idle_d / total_d)) as usize
    }
}

impl TryFrom<&str> for CpuCoreInfo {
    type Error = anyhow::Error;

    fn try_from(line: &str) -> Result<Self, Self::Error> {
        let parts = line.split(" ").collect::<Vec<_>>();
        let id = parts[0]
            .strip_prefix("cpu")
            .context("no 'cpu' prefix")?
            .parse()
            .context("non-int cpu")?;
        let times = parts[1..]
            .iter()
            .map(|n| n.parse::<usize>())
            .collect::<Result<Vec<_>, _>>()?;
        let idle = times[3];
        let total = times.iter().sum();
        Ok(Self { id, idle, total })
    }
}
