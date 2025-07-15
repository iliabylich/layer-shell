use anyhow::{Context as _, Result};
use module::Ctl;
use tokio::sync::mpsc::UnboundedSender;

pub struct TrayCtl {
    ctx: UnboundedSender<String>,
}

#[async_trait::async_trait]
impl Ctl for TrayCtl {
    const NAME: &str = "TrayCtl";

    type Command = String;

    fn new(ctx: UnboundedSender<Self::Command>) -> Self {
        Self { ctx }
    }

    async fn try_send(&self, command: Self::Command) -> Result<()> {
        self.ctx
            .send(command)
            .context("failed to trigger Tray; channel is closed")
    }
}
