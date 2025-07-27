use crate::{Ctl, TimerSubscriber};
use futures::Stream;
use std::time::Duration;
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
    time::sleep,
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_util::sync::CancellationToken;

#[async_trait::async_trait]
pub trait Module: Send {
    const NAME: &str;

    type Event: Send + 'static;
    type Command: Send + 'static;
    type Ctl: Ctl<Command = Self::Command>;

    fn spawn(
        token: CancellationToken,
        timer: TimerSubscriber,
    ) -> (impl Stream<Item = Self::Event>, JoinHandle<()>, Self::Ctl)
    where
        Self: Sized,
    {
        let (etx, erx) = tokio::sync::mpsc::unbounded_channel::<Self::Event>();
        let (ctx, crx) = tokio::sync::mpsc::unbounded_channel::<Self::Command>();

        let handle = tokio::task::spawn(async move {
            let mut task = Self::new(etx, crx, token.clone(), timer);

            loop {
                if let Err(err) = task.start().await {
                    log::error!(target: Self::NAME, "{err:?}");
                }

                if token.is_cancelled() {
                    log::error!(target: Self::NAME, "received exit signal, stopping...");
                    break;
                }

                sleep(Duration::from_secs(3)).await;
                log::error!(target: Self::NAME, "restarting...");
            }
        });

        (
            UnboundedReceiverStream::new(erx),
            handle,
            Self::Ctl::new(ctx),
        )
    }

    fn new(
        etx: UnboundedSender<Self::Event>,
        crx: UnboundedReceiver<Self::Command>,
        token: CancellationToken,
        timer: TimerSubscriber,
    ) -> Self
    where
        Self: Sized;

    async fn start(&mut self) -> anyhow::Result<()>;
}
