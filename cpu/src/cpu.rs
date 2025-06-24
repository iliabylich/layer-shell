use crate::{Event, store::Store};
use anyhow::Result;
use std::time::Duration;
use utils::{Emitter, service};

struct Task {
    emitter: Emitter<Event>,
    exit: tokio::sync::oneshot::Receiver<()>,
    timer: tokio::time::Interval,
    store: Store,
}

impl Task {
    async fn start(
        emitter: Emitter<Event>,
        exit: tokio::sync::oneshot::Receiver<()>,
    ) -> Result<()> {
        Self {
            emitter,
            exit,
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

                _ = &mut self.exit => {
                    log::info!(target: "CPU", "exiting...");
                    return Ok(())
                }
            }
        }
    }

    async fn tick(&mut self) -> Result<()> {
        let usage_per_core = self.store.update()?;

        self.emitter.emit(Event { usage_per_core }).await
    }
}

service!(CPU, Event, Task::start);
