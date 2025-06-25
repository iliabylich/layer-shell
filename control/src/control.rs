use crate::{dbus::DBus, event::Event};
use anyhow::Result;
use futures::Stream;
use pin_project_lite::pin_project;
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use zbus::Connection;

pin_project! {
    pub struct Control {
        #[pin]
        rx: UnboundedReceiver<Event>,
        #[pin]
        handle: JoinHandle<()>,
    }
}

impl Control {
    pub fn new(token: CancellationToken) -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Event>();
        let handle = tokio::task::spawn(async move {
            if let Err(err) = Self::r#loop(tx, token).await {
                log::error!("Control crashed: {err:?}");
            }
        });
        Self { rx, handle }
    }

    async fn r#loop(tx: UnboundedSender<Event>, token: CancellationToken) -> Result<()> {
        let connection = Connection::session().await?;
        let control = DBus::new(tx);
        connection.object_server().at("/Control", control).await?;
        connection.request_name("org.me.LayerShellControl").await?;

        token.cancelled().await;
        log::info!(target: "Control", "exiting...");

        Ok(())
    }
}

impl Stream for Control {
    type Item = Event;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut this = self.project();
        this.rx.poll_recv(cx)
    }
}

impl Future for Control {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if let Err(err) = futures::ready!(self.project().handle.poll(cx)) {
            log::error!("failed to await Control task: {err:?}")
        }
        std::task::Poll::Ready(())
    }
}
