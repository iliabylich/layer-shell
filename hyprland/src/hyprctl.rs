use anyhow::{Context as _, Result};
use module::Ctl;
use tokio::sync::mpsc::UnboundedSender;

pub struct Hyprctl {
    ctx: UnboundedSender<String>,
}

#[async_trait::async_trait]
impl Ctl for Hyprctl {
    const NAME: &str = "Hyprctl";

    type Command = String;

    fn new(ctx: UnboundedSender<Self::Command>) -> Self {
        Self { ctx }
    }

    async fn try_send(&self, command: Self::Command) -> Result<()> {
        self.ctx
            .send(command)
            .context("failed to send hyprctl command: channel is closed")
    }
}
