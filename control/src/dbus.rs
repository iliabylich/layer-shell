use crate::ControlEvent;
use tokio::sync::mpsc::UnboundedSender;
use zbus::interface;

pub(crate) struct DBus {
    sender: UnboundedSender<ControlEvent>,
}

impl DBus {
    pub(crate) fn new(sender: UnboundedSender<ControlEvent>) -> Self {
        Self { sender }
    }

    fn emit(&self, event: ControlEvent) {
        if let Err(err) = self.sender.send(event) {
            log::error!("{err:?}");
        }
    }
}

#[interface(name = "org.me.LayerShellControl")]
impl DBus {
    async fn toggle_session_screen(&self) {
        self.emit(ControlEvent::ToggleSessionScreen)
    }

    async fn reload_styles(&self) {
        self.emit(ControlEvent::ReloadStyles)
    }

    async fn exit(&self) {
        self.emit(ControlEvent::Exit)
    }
}
