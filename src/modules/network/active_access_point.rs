use crate::{
    modules::SystemDBus,
    utils::{
        StringRef, StringRefExt as _, dbus::infallible_property::InfalliblePropertyGetAndSubscribe,
    },
};
use dbus::{
    IncomingMessage, messages::network_manager::ActiveAccessPoint as ActiveAccessPointProperty,
};

pub(crate) struct ActiveAccessPoint {
    inner: InfalliblePropertyGetAndSubscribe<ActiveAccessPointProperty<StringRef>>,
}

#[derive(Debug)]
pub(crate) enum ActiveAccessPointEvent {
    Connected(StringRef),
    Disconnected,
}
impl From<&str> for ActiveAccessPointEvent {
    fn from(path: &str) -> Self {
        if path == "/" {
            Self::Disconnected
        } else {
            Self::Connected(StringRef::new(path))
        }
    }
}

impl ActiveAccessPoint {
    pub(crate) const fn new() -> Self {
        Self {
            inner: InfalliblePropertyGetAndSubscribe::new(SystemDBus::queue()),
        }
    }

    pub(crate) fn start(&mut self, path: StringRef) {
        self.inner
            .get_and_subscribe(ActiveAccessPointProperty::new(path));
    }

    pub(crate) fn stop(&mut self) {
        self.inner.unsubscribe();
    }

    pub(crate) fn handle(
        &mut self,
        message: IncomingMessage<'_>,
    ) -> Option<ActiveAccessPointEvent> {
        let path = self.inner.handle_reply_or_signal(message)?;
        Some(ActiveAccessPointEvent::from(path))
    }
}
