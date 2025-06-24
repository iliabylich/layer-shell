use crate::{dbus::DBus, event::Event};
use anyhow::Result;
use utils::{TaskCtx, service};
use zbus::Connection;

struct Task {
    ctx: TaskCtx<Event>,
}

impl Task {
    async fn start(ctx: TaskCtx<Event>) -> Result<()> {
        Self { ctx }.r#loop().await
    }

    async fn r#loop(self) -> Result<()> {
        let connection = Connection::session().await?;
        let control = DBus::new(self.ctx.emitter);
        connection.object_server().at("/Control", control).await?;
        connection.request_name("org.me.LayerShellControl").await?;

        self.ctx.exit.await?;
        Ok(())
    }
}

service!(Control, Event, Task::start);
