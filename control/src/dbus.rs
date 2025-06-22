use crate::Event;
use anyhow::Result;
use tokio::sync::mpsc::{Receiver, Sender};
use zbus::{Connection, interface};

pub(crate) struct DBus {
    tx: Sender<Event>,
}

impl DBus {
    pub(crate) async fn connect() -> Result<(Connection, Receiver<Event>)> {
        let (tx, rx) = tokio::sync::mpsc::channel(256);

        let connection = Connection::session().await?;
        let control = DBus { tx };
        connection.object_server().at("/Control", control).await?;
        connection.request_name("org.me.LayerShellControl").await?;

        Ok((connection, rx))
    }

    async fn send(&self, event: Event) {
        if self.tx.send(event).await.is_err() {
            log::error!("failed to send event, channel is closed");
        }
    }
}

#[interface(name = "org.me.LayerShellControl")]
impl DBus {
    async fn toggle_session_screen(&self) {
        self.send(Event::ToggleSessionScreen).await
    }

    async fn reload_styles(&self) {
        self.send(Event::ReloadStyles).await
    }

    async fn exit(&self) {
        self.send(Event::Exit).await
    }
}
