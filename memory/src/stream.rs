use crate::event::Event;
use anyhow::{Context as _, Result};
use futures_util::{Stream, ready};
use pin_project_lite::pin_project;
use std::{io::Read as _, task::Poll::*, time::Duration};
use tokio::time::{Interval, interval};

pin_project! {
    pub struct MemoryStream {
        #[pin]
        timer: Interval,

        buf: Vec<u8>,
    }
}

impl MemoryStream {
    pub fn new() -> Self {
        Self {
            timer: interval(Duration::from_secs(1)),
            buf: vec![0; 1_024],
        }
    }
}

impl Default for MemoryStream {
    fn default() -> Self {
        Self::new()
    }
}

impl Stream for MemoryStream {
    type Item = Event;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut this = self.project();

        let _ = ready!(this.timer.poll_tick(cx));

        match parse(this.buf) {
            Ok(event) => Ready(Some(event)),
            Err(err) => {
                log::error!("{err:?}");
                Ready(None)
            }
        }
    }
}

fn parse(buf: &mut [u8]) -> Result<Event> {
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

    Ok(Event {
        used: (used_kb as f64) / 1024.0 / 1024.0,
        total: (total_kb as f64) / 1024.0 / 1024.0,
    })
}
