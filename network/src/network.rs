use crate::{event::Event, network_task::NetworkTask};
use futures::Stream;
use pin_project_lite::pin_project;
use tokio::{sync::mpsc::UnboundedReceiver, task::JoinHandle};
use tokio_util::sync::CancellationToken;

pin_project! {
    pub struct Network {
        #[pin]
        rx: UnboundedReceiver<Event>,
        #[pin]
        handle: JoinHandle<()>,
    }
}

impl Network {
    pub fn new(token: CancellationToken) -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Event>();
        let handle = tokio::task::spawn(async move {
            if let Err(err) = NetworkTask::start(tx, token).await {
                log::error!("Network crashed: {err:?}");
            }
        });
        Self { rx, handle }
    }
}

impl Stream for Network {
    type Item = Event;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut this = self.project();
        this.rx.poll_recv(cx)
    }
}

impl Future for Network {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if let Err(err) = futures::ready!(self.project().handle.poll(cx)) {
            log::error!("failed to await Network task: {err:?}")
        }
        std::task::Poll::Ready(())
    }
}
