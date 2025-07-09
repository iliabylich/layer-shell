use crate::{SoundEvent, sound_task::SoundTask};
use futures::Stream;
use pin_project_lite::pin_project;
use tokio::{sync::mpsc::UnboundedReceiver, task::JoinHandle};
use tokio_util::sync::CancellationToken;

pin_project! {
    pub struct Sound {
        #[pin]
        rx: UnboundedReceiver<SoundEvent>
    }
}

const NAME: &str = "Sound";

impl Sound {
    pub fn new(token: CancellationToken) -> (&'static str, Self, JoinHandle<()>, ()) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<SoundEvent>();
        let handle = tokio::task::spawn(async move {
            if let Err(err) = SoundTask::start(tx, token).await {
                log::error!(target: "Network", "{err:?}");
            }
        });
        (NAME, Self { rx }, handle, ())
    }
}

impl Stream for Sound {
    type Item = SoundEvent;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.project().rx.poll_recv(cx)
    }
}
