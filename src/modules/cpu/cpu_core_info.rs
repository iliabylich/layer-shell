use anyhow::{Context as _, Result};
use tokio::{fs::File, io::AsyncReadExt as _};

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

    fn parse_line(line: &str) -> Result<Self> {
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
        let idle = times[3] + times[4];
        let total = times.iter().sum();
        Ok(Self { id, idle, total })
    }

    async fn parse_current() -> Result<Vec<Self>> {
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
                CpuCoreInfo::parse_line(line)
                    .with_context(|| format!("failed to parse line '{line}'"))
            })
            .collect::<Result<Vec<_>, _>>()
    }

    pub(crate) async fn parse_current_comparing_to(
        previous: &mut Option<Vec<CpuCoreInfo>>,
    ) -> Result<Vec<usize>> {
        let current = Self::parse_current().await?;
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
}
