use crate::{Event, dbus::DBus};
use anyhow::Result;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};
use zbus::Connection;

pub(crate) struct Task;

impl Task {
    pub(crate) fn spawn() -> (Receiver<Event>, JoinHandle<()>) {
        let (tx, rx) = tokio::sync::mpsc::channel(256);
        let handle = tokio::spawn(async move {
            if let Err(err) = Self::start(tx).await {
                log::error!("{err:?}");
            }
        });
        (rx, handle)
    }

    async fn start(tx: Sender<Event>) -> Result<()> {
        let connection = Connection::session().await?;
        let control = DBus::new(tx);
        connection.object_server().at("/Control", control).await?;
        connection.request_name("org.me.LayerShellControl").await?;
        loop {
            std::future::pending::<()>().await;
        }
    }
}
