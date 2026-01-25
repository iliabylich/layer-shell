use crate::dbus::{
    DBus, Message,
    messages::{
        body_is, interface_is, message_is,
        org_freedesktop_dbus::{AddMatch, RemoveMatch},
        type_is,
    },
    types::{CompleteType, Value},
};
use anyhow::Result;

pub(crate) trait SubscriptionResource {
    type Output;

    fn set_path(&mut self, path: String);
    fn try_process(&self, path: &str, interface: &str, items: &[Value]) -> Result<Self::Output>;
}

pub(crate) struct Subscription<S>
where
    S: SubscriptionResource,
{
    path: Option<String>,
    resource: S,
}

impl<S> Subscription<S>
where
    S: SubscriptionResource,
{
    pub(crate) fn new(resource: S) -> Self {
        Self {
            path: None,
            resource,
        }
    }

    fn unsubscribe(&mut self, dbus: &mut DBus) {
        let Some(old_path) = self.path.take() else {
            return;
        };

        let mut message: Message = RemoveMatch::new(&old_path).into();
        dbus.enqueue(&mut message);
    }

    fn subscribe(&mut self, dbus: &mut DBus, path: impl AsRef<str>) {
        let path = path.as_ref();
        let mut message: Message = AddMatch::new(path).into();
        dbus.enqueue(&mut message);
        self.path = Some(path.to_string());
        self.resource.set_path(path.to_string());
    }

    pub(crate) fn start(&mut self, dbus: &mut DBus, path: impl AsRef<str>) {
        self.unsubscribe(dbus);
        self.subscribe(dbus, path);
    }

    pub(crate) fn reset(&mut self, dbus: &mut DBus) {
        self.unsubscribe(dbus);
    }

    fn try_process(&self, message: &Message) -> Result<S::Output> {
        message_is!(
            message,
            Message::Signal {
                path,
                interface,
                body,
                ..
            }
        );

        interface_is!(interface, "org.freedesktop.DBus.Properties");
        body_is!(
            body,
            [Value::String(interface), Value::Array(item_t, items), _]
        );
        type_is!(item_t, CompleteType::DictEntry(key_t, value_t));
        type_is!(&**key_t, CompleteType::String);
        type_is!(&**value_t, CompleteType::Variant);

        self.resource.try_process(path, interface, items)
    }

    pub(crate) fn process(&self, message: &Message) -> Option<S::Output> {
        self.try_process(message).ok()
    }
}

impl<S> std::fmt::Debug for Subscription<S>
where
    S: SubscriptionResource,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subscription")
            .field("path", &self.path)
            .finish()
    }
}
