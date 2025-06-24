use crate::Event;
use utils::Emitter;
use zbus::interface;

pub(crate) struct DBus {
    emitter: Emitter<Event>,
}

impl DBus {
    pub(crate) fn new(emitter: Emitter<Event>) -> Self {
        Self { emitter }
    }

    fn emit(&self, event: Event) {
        if let Err(err) = self.emitter.emit(event) {
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
