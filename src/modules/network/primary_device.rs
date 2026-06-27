use crate::utils::{
    StringRef, StringRefExt as _,
    dbus::{infallible_property::InfalliblePropertyGetAndSubscribe, queue::DBusQueue},
};
use dbus::{IncomingMessage, messages::network_manager::PrimaryDevice as PrimaryDeviceProperty};

pub(crate) struct PrimaryDevice {
    inner: InfalliblePropertyGetAndSubscribe<PrimaryDeviceProperty<StringRef>>,
}

#[derive(Debug)]
pub(crate) enum PrimaryDeviceEvent {
    Connected(StringRef),
    Disconnected,
}
impl From<&str> for PrimaryDeviceEvent {
    fn from(path: &str) -> Self {
        if path == "/" {
            Self::Disconnected
        } else {
            Self::Connected(StringRef::new(path))
        }
    }
}

impl PrimaryDevice {
    pub(crate) const fn new() -> Self {
        Self {
            inner: InfalliblePropertyGetAndSubscribe::new(),
        }
    }

    pub(crate) fn start(&mut self, path: StringRef, q: &mut DBusQueue) {
        self.inner
            .get_and_subscribe(PrimaryDeviceProperty::new(path), q);
    }

    pub(crate) fn stop(&mut self, q: &mut DBusQueue) {
        self.inner.unsubscribe(q);
    }

    pub(crate) fn handle(
        &mut self,
        message: IncomingMessage<'_>,
        q: &mut DBusQueue,
    ) -> Option<PrimaryDeviceEvent> {
        let path = self.inner.handle_reply_or_signal(message, q)?;
        Some(PrimaryDeviceEvent::from(path))
    }
}
