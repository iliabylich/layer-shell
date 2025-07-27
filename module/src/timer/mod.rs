mod subscriber;
pub use subscriber::TimerSubscriber;

use anyhow::{Context as _, Result};
use std::time::Duration;
use tokio::{
    sync::broadcast::{Sender, channel},
    time::{Interval, interval},
};

pub struct Timer {
    interval: Interval,
    tx: Sender<u64>,
    state: u64,
}

impl Timer {
    pub fn new() -> Self {
        let (tx, _) = channel(256);
        let interval = interval(Duration::from_secs(1));

        Self {
            interval,
            tx,
            state: 0,
        }
    }

    pub fn tick(&mut self) -> Result<()> {
        log::info!("{}", self.state);
        self.tx
            .send(self.state)
            .context("failed to send tick information: channel is closed")?;

        self.state += 1;
        Ok(())
    }

    pub fn subscribe(&self) -> TimerSubscriber {
        let rx = self.tx.subscribe();
        TimerSubscriber::new(rx)
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

impl Future for Timer {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.interval.poll_tick(cx).map(|_| ())
    }
}
