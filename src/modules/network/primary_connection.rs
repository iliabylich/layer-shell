use crate::{
    modules::SystemDBus,
    utils::{
        StringRef, StringRefExt as _, dbus::infallible_property::InfalliblePropertyGetAndSubscribe,
    },
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
            inner: InfalliblePropertyGetAndSubscribe::new(SystemDBus::queue()),
        }
    }

    pub(crate) fn start(&mut self) {
        self.inner.get_and_subscribe(PrimaryConnectionProperty);
    }

    pub(crate) fn handle(
        &mut self,
        message: IncomingMessage<'_>,
    ) -> Option<PrimaryConnectionEvent> {
        let path = self.inner.handle_reply_or_signal(message)?;
        Some(PrimaryConnectionEvent::from(path))
    }
}
