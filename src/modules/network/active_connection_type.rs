use crate::{
    modules::SystemDBus,
    utils::{StringRef, dbus::infallible_property::InfalliblePropertyGetAndSubscribe},
};
use dbus::{
    IncomingMessage,
    messages::network_manager::ActiveConnectionType as ActiveConnectionTypeProperty,
};

pub(crate) struct ActiveConnectionType {
    inner: InfalliblePropertyGetAndSubscribe<ActiveConnectionTypeProperty<StringRef>>,
    path: Option<StringRef>,
}

impl ActiveConnectionType {
    pub(crate) const fn new() -> Self {
        Self {
            inner: InfalliblePropertyGetAndSubscribe::new(SystemDBus::queue()),
            path: None,
        }
    }

    pub(crate) fn start(&mut self, path: StringRef) {
        self.inner
            .get_and_subscribe(ActiveConnectionTypeProperty::new(path.clone()));
        self.path = Some(path);
    }

    pub(crate) fn stop(&mut self) {
        self.inner.unsubscribe();
        self.path = None;
    }

    pub(crate) fn handle(&mut self, message: IncomingMessage<'_>) -> Option<(bool, StringRef)> {
        let type_ = self.inner.handle_reply_or_signal(message)?;
        let is_wireless = type_.contains("wireless");
        let path = self.path.as_ref()?.clone();
        Some((is_wireless, path.clone()))
    }
}
