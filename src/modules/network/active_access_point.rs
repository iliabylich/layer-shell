use crate::utils::{
    StringRef, StringRefExt as _,
    dbus::{infallible_property::InfalliblePropertyGetAndSubscribe, queue::DBusQueue},
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
            inner: InfalliblePropertyGetAndSubscribe::new(),
        }
    }

    pub(crate) fn start(&mut self, path: StringRef, q: &mut DBusQueue) {
        self.inner
            .get_and_subscribe(ActiveAccessPointProperty::new(path), q);
    }

    pub(crate) fn stop(&mut self, q: &mut DBusQueue) {
        self.inner.unsubscribe(q);
    }

    pub(crate) fn handle(
        &mut self,
        message: IncomingMessage<'_>,
        q: &mut DBusQueue,
    ) -> Option<ActiveAccessPointEvent> {
        let path = self.inner.handle_reply_or_signal(message, q)?;
        Some(ActiveAccessPointEvent::from(path))
    }
}
