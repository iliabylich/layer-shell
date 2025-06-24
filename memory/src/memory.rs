use crate::event::Event;
use anyhow::{Context as _, Result};
use std::{io::Read as _, time::Duration};
use utils::{TaskCtx, service};

struct Task {
    ctx: TaskCtx<Event>,
    timer: tokio::time::Interval,
    buf: Vec<u8>,
}

impl Task {
    async fn start(ctx: TaskCtx<Event>) -> Result<()> {
        Self {
            ctx,
            timer: tokio::time::interval(Duration::from_secs(1)),
            buf: vec![0; 1_024],
        }
        .r#loop()
        .await
    }

    async fn r#loop(mut self) -> Result<()> {
        loop {
            tokio::select! {
                _ = self.timer.tick() => self.tick().await?,

                _ = &mut self.ctx.exit => {
                    log::info!(target: "Memory", "exiting...");
                    return Ok(())
                }
            }
        }
    }

    async fn tick(&mut self) -> Result<()> {
        let event = self.parse().await?;
        self.ctx.emitter.emit(event)
    }

    async fn parse(&mut self) -> Result<Event> {
        let mut file = std::fs::File::open("/proc/meminfo").context("failed to open")?;
        let len = file.read(&mut self.buf).context("failed to read")?;
        let contents = std::str::from_utf8(&self.buf[..len]).context("non-utf8 content")?;
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
}

service!(Memory, Event, Task::start);
