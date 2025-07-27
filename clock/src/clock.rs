use crate::ClockEvent;
use anyhow::{Context as _, Result};
use module::{Module, TimerSubscriber};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;

pub struct Clock {
    etx: UnboundedSender<ClockEvent>,
    token: CancellationToken,
    timer: TimerSubscriber,
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
        timer: TimerSubscriber,
    ) -> Self {
        Self {
            etx,
            token,
            timer: timer.with_cycle(1),
        }
    }

    async fn start(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                _ = self.timer.recv() => {
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
