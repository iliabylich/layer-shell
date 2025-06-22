use crate::Event;
use anyhow::{Context as _, Result};
use std::{io::Read as _, time::Duration};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

pub(crate) struct Task {
    tx: Sender<Event>,
}

impl Task {
    pub(crate) fn spawn() -> (Receiver<Event>, JoinHandle<()>) {
        let (tx, rx) = tokio::sync::mpsc::channel(256);

        let handle = tokio::spawn(async move {
            let task = Self { tx };
            if let Err(err) = task.start().await {
                log::error!("{err:?}");
            }
        });

        (rx, handle)
    }

    async fn start(&self) -> Result<()> {
        let mut buf = vec![0; 1_024];

        loop {
            let event = parse(&mut buf)?;
            self.tx
                .send(event)
                .await
                .context("failed to send event, channel is closed")?;

            tokio::time::sleep(Duration::from_secs(1)).await
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
