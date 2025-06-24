use crate::{dbus::DBus, event::Event};
use anyhow::Result;
use utils::{Emitter, service};
use zbus::Connection;

struct Task {
    emitter: Emitter<Event>,
    exit: tokio::sync::oneshot::Receiver<()>,
}

impl Task {
    async fn start(
        emitter: Emitter<Event>,
        exit: tokio::sync::oneshot::Receiver<()>,
    ) -> Result<()> {
        Self { emitter, exit }.r#loop().await
    }

    async fn r#loop(self) -> Result<()> {
        let connection = Connection::session().await?;
        let control = DBus::new(self.emitter);
        connection.object_server().at("/Control", control).await?;
        connection.request_name("org.me.LayerShellControl").await?;

        self.exit.await?;
        Ok(())
    }
}

service!(Control, Event, Task::start);
