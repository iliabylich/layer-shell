use crate::Event;
use tokio::sync::mpsc::Sender;
use zbus::interface;

pub(crate) struct DBus {
    tx: Sender<Event>,
}

impl DBus {
    pub(crate) fn new(tx: Sender<Event>) -> Self {
        Self { tx }
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
