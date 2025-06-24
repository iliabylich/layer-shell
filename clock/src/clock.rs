use crate::Event;
use anyhow::Result;
use std::time::Duration;
use utils::{TaskCtx, service};

struct Task {
    ctx: TaskCtx<Event>,
    timer: tokio::time::Interval,
}

impl Task {
    async fn start(ctx: TaskCtx<Event>) -> Result<()> {
        Self {
            ctx,
            timer: tokio::time::interval(Duration::from_secs(1)),
        }
        .r#loop()
        .await
    }

    async fn r#loop(mut self) -> Result<()> {
        loop {
            tokio::select! {
                _ = self.timer.tick() => self.tick()?,

                _ = &mut self.ctx.exit => {
                    log::info!(target: "Clock", "exiting...");
                    return Ok(())
                },
            }
        }
    }

    fn tick(&self) -> Result<()> {
        let time = chrono::Local::now()
            .format("%H:%M:%S | %b %e | %a")
            .to_string();
        self.ctx.emitter.emit(Event { time })
    }
}

service!(Clock, Event, Task::start);
