use tokio::sync::mpsc::UnboundedSender;

#[async_trait::async_trait]
pub trait Ctl {
    const NAME: &str;

    type Command: Send + 'static;

    fn new(ctx: UnboundedSender<Self::Command>) -> Self
    where
        Self: Sized;

    async fn send(&self, command: Self::Command) {
        if let Err(err) = self.try_send(command).await {
            log::error!(target: Self::NAME, "{err:?}");
        }
    }

    async fn try_send(&self, command: Self::Command) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
impl Ctl for () {
    const NAME: &str = "NoCtl";

    type Command = ();

    fn new(_: UnboundedSender<Self::Command>) -> Self {}

    async fn try_send(&self, _: Self::Command) -> anyhow::Result<()> {
        Ok(())
    }
}
