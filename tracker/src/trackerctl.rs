use crate::command::TrackerCommand;
use anyhow::{Context as _, Result};
use async_trait::async_trait;
use module::Ctl;
use tokio::sync::mpsc::UnboundedSender;

pub struct TrackerCtl {
    ctx: UnboundedSender<TrackerCommand>,
}

#[async_trait]
impl Ctl for TrackerCtl {
    const NAME: &str = "TrackerCtl";

    type Command = TrackerCommand;

    fn new(ctx: UnboundedSender<Self::Command>) -> Self {
        Self { ctx }
    }

    async fn try_send(&self, command: Self::Command) -> Result<()> {
        self.ctx
            .send(command)
            .context("failed to trigger Tray; channel is closed")
    }
}

impl TrackerCtl {
    pub async fn toggle(&self) {
        self.send(TrackerCommand::Toggle).await
    }

    pub async fn add(&self, title: String) {
        self.send(TrackerCommand::Add { title }).await
    }

    pub async fn remove(&self, uuid: String) {
        self.send(TrackerCommand::Remove { uuid }).await
    }

    pub async fn select(&self, uuid: String) {
        self.send(TrackerCommand::Select { uuid }).await
    }

    pub async fn cut(&self) {
        self.send(TrackerCommand::Cut).await
    }
}
