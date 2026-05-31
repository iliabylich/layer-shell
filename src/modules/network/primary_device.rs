use crate::{
    modules::SystemDBus,
    utils::{
        StringRef, StringRefExt as _, dbus::infallible_property::InfalliblePropertyGetAndSubscribe,
    },
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
            inner: InfalliblePropertyGetAndSubscribe::new(SystemDBus::queue()),
        }
    }

    pub(crate) fn start(&mut self, path: StringRef) {
        self.inner
            .get_and_subscribe(PrimaryDeviceProperty::new(path));
    }

    pub(crate) fn stop(&mut self) {
        self.inner.unsubscribe();
    }

    pub(crate) fn handle(&mut self, message: IncomingMessage<'_>) -> Option<PrimaryDeviceEvent> {
        let path = self.inner.handle_reply_or_signal(message)?;
        Some(PrimaryDeviceEvent::from(path))
    }
}
