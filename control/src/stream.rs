use crate::{dbus::DBus, event::Event};
use anyhow::Result;
use futures_util::Stream;
use pin_project_lite::pin_project;
use tokio::sync::mpsc::Receiver;
use zbus::Connection;

pin_project! {
    pub struct ControlStream {
        connection: Connection,

        #[pin]
        rx: Receiver<Event>
    }
}

impl ControlStream {
    pub async fn new() -> Result<Self> {
        let (connection, rx) = DBus::connect().await?;
        Ok(Self { connection, rx })
    }
}

impl Stream for ControlStream {
    type Item = Event;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut this = self.project();
        this.rx.poll_recv(cx)
    }
}
