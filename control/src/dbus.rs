use crate::Event;
use tokio::sync::mpsc::UnboundedSender;
use zbus::interface;

pub(crate) struct DBus {
    sender: UnboundedSender<Event>,
}

impl DBus {
    pub(crate) fn new(sender: UnboundedSender<Event>) -> Self {
        Self { sender }
    }

    fn emit(&self, event: Event) {
        if let Err(err) = self.sender.send(event) {
            log::error!("{err:?}");
        }
    }
}

#[interface(name = "org.me.LayerShellControl")]
impl DBus {
    async fn toggle_session_screen(&self) {
        self.emit(Event::ToggleSessionScreen)
    }

    async fn reload_styles(&self) {
        self.emit(Event::ReloadStyles)
    }

    async fn exit(&self) {
        self.emit(Event::Exit)
    }
}
