use crate::Event;
use anyhow::Result;
use std::time::Duration;
use utils::{Emitter, service};

struct Task {
    emitter: Emitter<Event>,
    exit: tokio::sync::oneshot::Receiver<()>,
    timer: tokio::time::Interval,
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
        }
        .r#loop()
        .await
    }

    async fn r#loop(mut self) -> Result<()> {
        loop {
            tokio::select! {
                _ = self.timer.tick() => self.tick().await?,

                _ = &mut self.exit => {
                    log::info!(target: "Clock", "exiting...");
                    return Ok(())
                },
            }
        }
    }

    async fn tick(&self) -> Result<()> {
        let time = chrono::Local::now()
            .format("%H:%M:%S | %b %e | %a")
            .to_string();
        self.emitter.emit(Event { time }).await
    }
}

service!(Clock, Event, Task::start);
