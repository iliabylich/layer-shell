use crate::MemoryEvent;
use anyhow::{Context as _, Result};
use futures::{Stream, ready};
use pin_project_lite::pin_project;
use std::{io::Read as _, time::Duration};
use tokio::time::{Interval, interval};

pin_project! {
    pub struct Memory {
        #[pin]
        timer: Interval,
        buf: Vec<u8>,
    }
}

impl Memory {
    pub fn new() -> Self {
        Self {
            timer: interval(Duration::from_secs(1)),
            buf: vec![0; 1_024],
        }
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl Stream for Memory {
    type Item = MemoryEvent;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut this = self.project();
        let _ = ready!(this.timer.poll_tick(cx));

        match parse(this.buf) {
            Ok(event) => std::task::Poll::Ready(Some(event)),
            Err(err) => {
                log::error!("Memory stream has crashes: {err:?}");
                std::task::Poll::Ready(None)
            }
        }
    }
}

fn parse(buf: &mut [u8]) -> Result<MemoryEvent> {
    let mut file = std::fs::File::open("/proc/meminfo").context("failed to open")?;
    let len = file.read(buf).context("failed to read")?;
    let contents = std::str::from_utf8(&buf[..len]).context("non-utf8 content")?;
    let mut lines = contents.lines();

    let parse_mem = |line: &str, prefix: &str| {
        line.trim_ascii_end()
            .strip_prefix(prefix)
            .with_context(|| format!("no {prefix} prefix"))?
            .strip_suffix("kB")
            .context("no 'kB' suffix")?
            .trim_ascii()
            .parse::<usize>()
            .with_context(|| format!("not an int on {prefix} line"))
    };

    let line1 = lines.next().context("no line 1")?;
    let total_kb = parse_mem(line1, "MemTotal:")?;

    let _line2 = lines.next().context("no line 2")?;

    let line3 = lines.next().context("no line 3")?;
    let available_kb = parse_mem(line3, "MemAvailable:")?;

    let used_kb = total_kb - available_kb;

    Ok(MemoryEvent {
        used: (used_kb as f64) / 1024.0 / 1024.0,
        total: (total_kb as f64) / 1024.0 / 1024.0,
    })
}
