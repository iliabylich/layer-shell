use anyhow::{Context as _, Result};
use tokio::sync::broadcast::Receiver;

pub struct TimerSubscriber {
    rx: Receiver<u64>,
    cycle: Option<u64>,
}

impl TimerSubscriber {
    pub(crate) fn new(rx: Receiver<u64>) -> Self {
        Self { rx, cycle: None }
    }

    pub fn with_cycle(mut self, cycle: u64) -> Self {
        self.cycle = Some(cycle);
        self
    }

    pub async fn recv(&mut self) -> Result<u64> {
        let cycle = self.cycle.context("TimerSubscriber has no cycle")?;

        loop {
            let time = self
                .rx
                .recv()
                .await
                .context("failed to recv from inner timer chennel")?;

            if time % cycle == 0 {
                return Ok(time);
            }
        }
    }
}
