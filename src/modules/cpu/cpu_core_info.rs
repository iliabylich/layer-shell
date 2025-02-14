use anyhow::{Context as _, Result};
use std::io::Read as _;

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
        let mut id = 0;
        let mut idle = 0;
        let mut total = 0;
        for (idx, part) in line.split(" ").enumerate() {
            if idx == 0 {
                id = part
                    .strip_prefix("cpu")
                    .context("no 'cpu' prefix")?
                    .parse()
                    .context("non-int cpu")?;
                continue;
            }
            let num = part.parse::<usize>()?;
            total += num;
            if idx == 4 || idx == 5 {
                idle += num;
            }
        }
        Ok(Self { id, idle, total })
    }

    fn parse_current(buf: &mut [u8]) -> Result<Vec<Self>> {
        let mut file = std::fs::File::open("/proc/stat").context("failed to open")?;
        let len = file.read(buf).context("failed to read")?;
        let contents = std::str::from_utf8(&buf[..len]).context("non-utf8 content")?;

        contents
            .lines()
            .filter(|line| line.starts_with("cpu") && line.as_bytes()[3].is_ascii_digit())
            .map(|line| {
                CpuCoreInfo::parse_line(line)
                    .with_context(|| format!("failed to parse line '{line}'"))
            })
            .collect::<Result<Vec<_>, _>>()
    }

    pub(crate) fn parse_current_comparing_to(
        previous: Option<&Vec<CpuCoreInfo>>,
        buf: &mut [u8],
    ) -> Result<(Vec<usize>, Vec<CpuCoreInfo>)> {
        let current = Self::parse_current(buf)?;
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
