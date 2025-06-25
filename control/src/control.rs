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
        handle: JoinHandle<()>,
        token: CancellationToken,
    }
}

impl Control {
    pub fn new() -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Event>();
        let token = CancellationToken::new();

        let handle = {
            let token = token.clone();
            tokio::task::spawn(async move {
                if let Err(err) = Self::r#loop(tx, token).await {
                    log::error!("Control crashed: {err:?}");
                }
            })
        };

        Self { rx, handle, token }
    }

    pub async fn stop(self) -> Result<()> {
        self.token.cancel();
        self.handle.await?;
        Ok(())
    }

    async fn r#loop(tx: UnboundedSender<Event>, token: CancellationToken) -> Result<()> {
        let connection = Connection::session().await?;
        let control = DBus::new(tx);
        connection.object_server().at("/Control", control).await?;
        connection.request_name("org.me.LayerShellControl").await?;

        token.cancelled().await;

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
