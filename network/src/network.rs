use crate::{event::NetworkEvent, network_task::NetworkTask};
use futures::Stream;
use pin_project_lite::pin_project;
use tokio::{sync::mpsc::UnboundedReceiver, task::JoinHandle};
use tokio_util::sync::CancellationToken;

pin_project! {
    pub struct Network {
        #[pin]
        rx: UnboundedReceiver<NetworkEvent>,
    }
}

const NAME: &str = "Network";

impl Network {
    pub fn new(token: CancellationToken) -> (&'static str, Self, JoinHandle<()>, ()) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<NetworkEvent>();
        let handle = tokio::task::spawn(async move {
            if let Err(err) = NetworkTask::start(tx, token).await {
                log::error!("{NAME} crashed: {err:?}");
            }
        });
        (NAME, Self { rx }, handle, ())
    }
}

impl Stream for Network {
    type Item = NetworkEvent;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut this = self.project();
        this.rx.poll_recv(cx)
    }
}
