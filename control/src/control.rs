use crate::{ControlEvent, dbus::DBus};
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
        rx: UnboundedReceiver<ControlEvent>,
    }
}

const NAME: &str = "Control";

impl Control {
    pub fn new(token: CancellationToken) -> (&'static str, Self, JoinHandle<()>) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<ControlEvent>();
        let handle = tokio::task::spawn(async move {
            if let Err(err) = Self::r#loop(tx, token).await {
                log::error!("{NAME} crashed: {err:?}");
            }
        });
        (NAME, Self { rx }, handle)
    }

    async fn r#loop(tx: UnboundedSender<ControlEvent>, token: CancellationToken) -> Result<()> {
        let connection = Connection::session().await?;
        let control = DBus::new(tx);
        connection.object_server().at("/Control", control).await?;
        connection.request_name("org.me.LayerShellControl").await?;

        token.cancelled().await;
        log::info!(target: NAME, "exiting...");

        Ok(())
    }
}

impl Stream for Control {
    type Item = ControlEvent;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut this = self.project();
        this.rx.poll_recv(cx)
    }
}
