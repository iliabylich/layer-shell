use crate::utils::{
    StringRef,
    dbus::{infallible_property::InfalliblePropertyGetAndSubscribe, queue::DBusQueue},
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
            inner: InfalliblePropertyGetAndSubscribe::new(),
            path: None,
        }
    }

    pub(crate) fn start(&mut self, path: StringRef, q: &mut DBusQueue) {
        self.inner
            .get_and_subscribe(ActiveConnectionTypeProperty::new(path.clone()), q);
        self.path = Some(path);
    }

    pub(crate) fn stop(&mut self, q: &mut DBusQueue) {
        self.inner.unsubscribe(q);
        self.path = None;
    }

    pub(crate) fn handle(
        &mut self,
        message: IncomingMessage<'_>,
        q: &mut DBusQueue,
    ) -> Option<(bool, StringRef)> {
        let type_ = self.inner.handle_reply_or_signal(message, q)?;
        let is_wireless = type_.contains("wireless");
        let path = self.path.as_ref()?.clone();
        Some((is_wireless, path.clone()))
    }
}
