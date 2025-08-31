use crate::{
    TrackerCtl, TrackerEvent, TrackerUpdatedEvent, command::TrackerCommand, disk::Disk,
    state::State,
};
use anyhow::{Context as _, Result};
use async_trait::async_trait;
use module::{Module, TimerSubscriber};
use std::str::FromStr;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct Tracker {
    etx: UnboundedSender<TrackerEvent>,
    crx: UnboundedReceiver<TrackerCommand>,
    token: CancellationToken,
    timer: TimerSubscriber,
}

#[async_trait]
impl Module for Tracker {
    const NAME: &str = "Tracker";

    type Event = TrackerEvent;
    type Command = TrackerCommand;
    type Ctl = TrackerCtl;

    fn new(
        etx: UnboundedSender<Self::Event>,
        crx: UnboundedReceiver<Self::Command>,
        token: CancellationToken,
        timer: TimerSubscriber,
    ) -> Self {
        Self {
            etx,
            crx,
            token,
            timer: timer.with_cycle(1),
        }
    }

    async fn start(&mut self) -> Result<()> {
        let disk = Disk::new().await?;
        let mut state = disk.read_latest_state().await?;
        state.stop();

        loop {
            tokio::select! {
                Some(cmd) = self.crx.recv() => {
                    self.on_command(&mut state, &disk, cmd).await?;
                }

                Ok(_) = self.timer.recv() => {
                    self.send(&state);
                }

                _ = self.token.cancelled() => {
                    log::info!(target: "Tracker", "exiting...");
                    return Ok(())
                }
            }
        }
    }
}

impl Tracker {
    async fn on_command(
        &self,
        state: &mut State,
        disk: &Disk,
        command: TrackerCommand,
    ) -> Result<()> {
        match command {
            TrackerCommand::Toggle => state.toggle(),
            TrackerCommand::Add { title } => state.add(title),
            TrackerCommand::Remove { uuid } => {
                let uuid = Uuid::from_str(&uuid).context("failed to parse UUID")?;
                state.remove(uuid);
            }
            TrackerCommand::Select { uuid } => {
                let uuid = Uuid::from_str(&uuid).context("failed to parse UUID")?;
                state.select(uuid);
            }
            TrackerCommand::Cut => *state = State::empty(),
        }

        disk.write(state).await?;

        self.send(state);

        Ok(())
    }

    fn send(&self, state: &State) {
        if self
            .etx
            .send(TrackerEvent::Updated(TrackerUpdatedEvent {
                view: state.serialize(),
            }))
            .is_err()
        {
            log::error!("failed to send tracker event: channel is closed");
        }
    }
}
