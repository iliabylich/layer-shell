use crate::utils::{
    StringRef, StringRefExt as _,
    dbus::{infallible_property::InfalliblePropertyGetAndSubscribe, queue::SystemDBusQueue},
};
use dbus::{
    IncomingMessage, messages::network_manager::PrimaryConnection as PrimaryConnectionProperty,
};

pub(crate) struct PrimaryConnection {
    inner: InfalliblePropertyGetAndSubscribe<PrimaryConnectionProperty>,
}

pub(crate) enum PrimaryConnectionEvent {
    Connected(StringRef),
    Disconnected,
}

impl From<&str> for PrimaryConnectionEvent {
    fn from(path: &str) -> Self {
        if path == "/" {
            Self::Disconnected
        } else {
            Self::Connected(StringRef::new(path))
        }
    }
}

impl PrimaryConnection {
    pub(crate) const fn new() -> Self {
        Self {
            inner: InfalliblePropertyGetAndSubscribe::new(),
        }
    }

    pub(crate) fn start(&mut self, q: &mut SystemDBusQueue) {
        self.inner.get_and_subscribe(PrimaryConnectionProperty, q);
    }

    pub(crate) fn handle(
        &mut self,
        message: IncomingMessage<'_>,
        q: &mut SystemDBusQueue,
    ) -> Option<PrimaryConnectionEvent> {
        let path = self.inner.handle_reply_or_signal(message, q)?;
        Some(PrimaryConnectionEvent::from(path))
    }
}
