use crate::{CpuUsageEvent, store::Store};
use anyhow::{Context as _, Result};
use module::Module;
use std::time::Duration;
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    time::interval,
};
use tokio_util::sync::CancellationToken;

pub struct CPU {
    etx: UnboundedSender<CpuUsageEvent>,
    token: CancellationToken,
}

#[async_trait::async_trait]
impl Module for CPU {
    const NAME: &str = "CPU";

    type Event = CpuUsageEvent;
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
        let mut timer = interval(Duration::from_secs(1));
        let mut store = Store::new();

        loop {
            tokio::select! {
                _ = timer.tick() => {
                    self.tick(&mut store).await?;
                }

                _ = self.token.cancelled() => {
                    log::info!(target: "CPU", "exiting...");
                    return Ok(())
                }
            }
        }
    }
}

impl CPU {
    async fn tick(&self, store: &mut Store) -> Result<()> {
        let usage_per_core = store.update()?;
        let event = CpuUsageEvent {
            usage_per_core: usage_per_core.into(),
        };
        self.etx
            .send(event)
            .context("failed to send CpuUsageEvent: channel is closed")?;
        Ok(())
    }
}
