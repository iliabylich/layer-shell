use crate::MemoryEvent;
use anyhow::{Context as _, Result};
use module::Module;
use std::{io::Read as _, time::Duration};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;

pub struct Memory {
    etx: UnboundedSender<MemoryEvent>,
    token: CancellationToken,
}

#[async_trait::async_trait]
impl Module for Memory {
    const NAME: &str = "Memory";

    type Event = MemoryEvent;
    type Command = ();
    type Ctl = ();

    fn new(
        etx: UnboundedSender<Self::Event>,
        _: UnboundedReceiver<Self::Command>,
        token: CancellationToken,
    ) -> Self {
        Self { etx, token }
    }

    async fn start(&mut self) -> Result<()> {
        let mut timer = tokio::time::interval(Duration::from_secs(1));
        let mut buf = vec![0; 1_024];

        loop {
            tokio::select! {
                _ = timer.tick() => {
                    self.tick(&mut buf)?;
                }

                _ = self.token.cancelled() => {
                    log::info!(target: "Clock", "exiting...");
                    return Ok(())
                }
            }
        }
    }
}

impl Memory {
    fn tick(&self, buf: &mut [u8]) -> Result<()> {
        let event = parse(buf)?;
        self.etx
            .send(event)
            .context("failed to send MemoryEvent: channel is closed")
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
