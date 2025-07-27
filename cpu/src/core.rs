use anyhow::{Context as _, Result};
use std::io::SeekFrom;
use tokio::{
    fs::File,
    io::{AsyncReadExt as _, AsyncSeekExt as _},
};

#[derive(Debug, Clone)]
pub(crate) struct Core {
    pub(crate) id: u8,
    pub(crate) idle: u64,
    pub(crate) total: u64,
}

impl Core {
    fn parse_line(line: &str) -> Result<Self> {
        let mut parts = line.split(" ");

        fn cut_str<'a>(
            i: &mut impl Iterator<Item = &'a str>,
            idx: usize,
            name: &str,
        ) -> Result<&'a str> {
            i.next()
                .with_context(|| format!("no {idx} item ({name}) in CPU line"))
        }

        fn cut_u64<'a>(
            i: &mut impl Iterator<Item = &'a str>,
            idx: usize,
            name: &str,
        ) -> Result<u64> {
            let s = cut_str(i, idx, name)?;
            s.parse()
                .with_context(|| format!("non-int {name} component: {s}"))
        }

        let id = cut_str(&mut parts, 0, "cpuN")?
            .strip_prefix("cpu")
            .context("no 'cpu' prefix in CPU line")?
            .parse::<u8>()
            .context("non-int CPU")?;

        let user_n = cut_u64(&mut parts, 1, "user")?;
        let nice_n = cut_u64(&mut parts, 2, "nice")?;
        let system_n = cut_u64(&mut parts, 3, "system")?;
        let idle_n = cut_u64(&mut parts, 4, "idle")?;
        let iowait_n = cut_u64(&mut parts, 5, "iowait")?;
        let irq_n = cut_u64(&mut parts, 6, "irq")?;
        let softirq_n = cut_u64(&mut parts, 7, "softirq")?;
        let steal_n = cut_u64(&mut parts, 8, "steal")?;
        let guest_n = cut_u64(&mut parts, 9, "guest")?;
        let guest_nice_n = cut_u64(&mut parts, 10, "guest_nice")?;

        let idle = idle_n + iowait_n;
        let total = user_n
            + nice_n
            + system_n
            + idle_n
            + iowait_n
            + irq_n
            + softirq_n
            + steal_n
            + guest_n
            + guest_nice_n;

        Ok(Self { id, idle, total })
    }

    pub(crate) async fn read_and_parse_all(f: &mut File, buf: &mut [u8]) -> Result<Vec<Self>> {
        f.seek(SeekFrom::Start(0))
            .await
            .context("failed to fseek")?;
        let len = f.read(buf).await.context("failed to read")?;
        let contents = std::str::from_utf8(&buf[..len]).context("non-utf8 content")?;

        contents
            .lines()
            .filter(|line| line.starts_with("cpu") && line.as_bytes()[3].is_ascii_digit())
            .map(|line| {
                Self::parse_line(line).with_context(|| format!("failed to parse line '{line}'"))
            })
            .collect::<Result<Vec<_>, _>>()
    }
}
