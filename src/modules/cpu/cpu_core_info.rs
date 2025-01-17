use anyhow::{Context as _, Result};

#[derive(Debug, Clone)]
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

    fn parse_current() -> Result<Vec<Self>> {
        let contents =
            std::fs::read_to_string("/proc/stat").context("failed to read /proc/stat")?;

        contents
            .split("\n")
            .filter(|line| line.starts_with("cpu") && line.as_bytes()[3].is_ascii_digit())
            .map(|line| {
                CpuCoreInfo::parse_line(line)
                    .with_context(|| format!("failed to parse line '{line}'"))
            })
            .collect::<Result<Vec<_>, _>>()
    }

    pub(crate) fn parse_current_comparing_to(
        previous: Option<Vec<CpuCoreInfo>>,
    ) -> Result<(Vec<usize>, Vec<CpuCoreInfo>)> {
        let current = Self::parse_current()?;
        let count = current.len();

        let usage = if let Some(previous) = previous {
            assert_eq!(previous.len(), current.len());

            previous
                .iter()
                .zip(current.iter())
                .map(|(prev, next)| next.load_comparing_to(prev))
                .collect::<Vec<_>>()
        } else {
            vec![0; count]
        };

        Ok((usage, current))
    }
}
