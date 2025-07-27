use crate::{ControlEvent, dbus::DBus};
use module::{Module, TimerSubscriber};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;
use zbus::Connection;

pub struct Control {
    etx: UnboundedSender<ControlEvent>,
    token: CancellationToken,
}

#[async_trait::async_trait]
impl Module for Control {
    const NAME: &str = "Control";

    type Event = ControlEvent;
    type Command = ();
    type Ctl = ();

    fn new(
        etx: UnboundedSender<Self::Event>,
        _: UnboundedReceiver<Self::Command>,
        token: CancellationToken,
        _: TimerSubscriber,
    ) -> Self {
        Self { etx, token }
    }

    async fn start(&mut self) -> anyhow::Result<()> {
        let connection = Connection::session().await?;
        let control = DBus::new(self.etx.clone());
        connection.object_server().at("/Control", control).await?;
        connection.request_name("org.me.LayerShellControl").await?;

        self.token.cancelled().await;
        log::info!(target: "Control", "exiting...");

        Ok(())
    }
}
