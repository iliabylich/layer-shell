use crate::{Event, store::Store};
use anyhow::Result;
use std::time::Duration;
use utils::{TaskCtx, service};

struct Task {
    ctx: TaskCtx<Event>,
    timer: tokio::time::Interval,
    store: Store,
}

impl Task {
    async fn start(ctx: TaskCtx<Event>) -> Result<()> {
        Self {
            ctx,
            timer: tokio::time::interval(Duration::from_secs(1)),
            store: Store::new(),
        }
        .r#loop()
        .await
    }

    async fn r#loop(mut self) -> Result<()> {
        loop {
            tokio::select! {
                _ = self.timer.tick() => self.tick().await?,

                _ = &mut self.ctx.exit => {
                    log::info!(target: "CPU", "exiting...");
                    return Ok(())
                }
            }
        }
    }

    async fn tick(&mut self) -> Result<()> {
        let usage_per_core = self.store.update()?;

        self.ctx.emitter.emit(Event { usage_per_core }).await
    }
}

service!(CPU, Event, Task::start);
