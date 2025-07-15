use crate::ClockEvent;
use anyhow::{Context as _, Result};
use module::Module;
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;

pub struct Clock {
    etx: UnboundedSender<ClockEvent>,
    token: CancellationToken,
}

#[async_trait::async_trait]
impl Module for Clock {
    const NAME: &str = "Clock";

    type Event = ClockEvent;
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

        loop {
            tokio::select! {
                _ = timer.tick() => {
                    self.tick()?;
                }

                _ = self.token.cancelled() => {
                    log::info!(target: "Clock", "exiting...");
                    return Ok(())
                }
            }
        }
    }
}

impl Clock {
    fn tick(&self) -> Result<()> {
        let time = chrono::Local::now()
            .format("%H:%M:%S | %b %e | %a")
            .to_string();
        self.etx
            .send(ClockEvent { time: time.into() })
            .context("failed to send ClockEvent: channel is closed")
    }
}
