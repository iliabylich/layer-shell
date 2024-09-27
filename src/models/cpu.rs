use crate::utils::singleton;
use anyhow::{Context, Result};
use tokio::{fs::File, io::AsyncReadExt};

pub(crate) struct CPU {
    previous: Option<Vec<CpuCoreInfo>>,
    callbacks: Vec<fn(CPUData)>,
}
singleton!(CPU);

impl CPU {
    pub(crate) async fn spawn() {
        Self::set(Self {
            previous: None,
            callbacks: vec![],
        });

        loop {
            match parse().await {
                Ok(data) => Self::changed(data),
                Err(err) => {
                    eprintln!("failed to read system CPU usage:\n{}", err)
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }

    pub(crate) fn subscribe(f: fn(CPUData)) {
        this().callbacks.push(f);
    }

    fn changed(data: CPUData) {
        for callback in this().callbacks.iter() {
            (callback)(data.clone());
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CPUData {
    pub(crate) cores: Vec<usize>,
}

async fn parse() -> Result<CPUData> {
    let current = parse_current().await?;
    let count = current.len();

    if let Some(previous) = this().previous.take() {
        assert_eq!(previous.len(), current.len());

        let usage = previous
            .iter()
            .zip(current.iter())
            .map(|(prev, next)| next.load_comparing_to(prev))
            .collect::<Vec<_>>();

        this().previous = Some(current);

        Ok(CPUData { cores: usage })
    } else {
        this().previous = Some(current);
        Ok(CPUData {
            cores: vec![0; count],
        })
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
