use crate::{TrayEvent, tray_task::TrayTask};
use futures::Stream;
use pin_project_lite::pin_project;
use tokio::{sync::mpsc::UnboundedReceiver, task::JoinHandle};
use tokio_util::sync::CancellationToken;

pin_project! {
    pub struct Tray {
        #[pin]
        rx: UnboundedReceiver<TrayEvent>
    }
}

const NAME: &str = "Tray";

impl Tray {
    pub fn new(token: CancellationToken) -> (&'static str, Self, JoinHandle<()>) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<TrayEvent>();
        let handle = tokio::task::spawn(async move {
            if let Err(err) = TrayTask::start(tx, token).await {
                log::error!("{NAME} crashed: {err:?}");
            }
        });
        (NAME, Self { rx }, handle)
    }
}

impl Stream for Tray {
    type Item = TrayEvent;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.project().rx.poll_recv(cx)
    }
}
