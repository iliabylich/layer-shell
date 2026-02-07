use crate::dbus::{
    DBus, Message, Oneshot, OneshotResource, Subscription, SubscriptionResource,
    messages::{body_is, interface_is, org_freedesktop_dbus::GetProperty, path_is, value_is},
    types::Value,
};
use anyhow::{Result, bail};

#[derive(Debug)]
pub(crate) struct PrimaryConnection {
    oneshot: Oneshot<Resource>,
    subscription: Subscription<Resource>,
}

pub(crate) enum PrimaryConnectionEvent {
    Connected(String),
    Disconnected,
}

impl From<String> for PrimaryConnectionEvent {
    fn from(path: String) -> Self {
        if path == "/" {
            PrimaryConnectionEvent::Disconnected
        } else {
            PrimaryConnectionEvent::Connected(path)
        }
    }
}

impl PrimaryConnection {
    pub(crate) fn new() -> Self {
        Self {
            oneshot: Oneshot::new(Resource),
            subscription: Subscription::new(Resource),
        }
    }

    pub(crate) fn init(&mut self, dbus: &mut DBus) {
        self.oneshot.start(dbus, ());
        self.subscription
            .start(dbus, "/org/freedesktop/NetworkManager");
    }

    pub(crate) fn on_message(&mut self, message: &Message) -> Option<PrimaryConnectionEvent> {
        None.or_else(|| self.oneshot.process(message))
            .or_else(|| self.subscription.process(message))
            .map(PrimaryConnectionEvent::from)
    }
}

struct Resource;
impl OneshotResource for Resource {
    type Input = ();
    type Output = String;

    fn make_request(&self, _input: Self::Input) -> Message<'static> {
        GetProperty::new(
            "org.freedesktop.NetworkManager",
            "/org/freedesktop/NetworkManager",
            "org.freedesktop.NetworkManager",
            "PrimaryConnection",
        )
        .into()
    }

    fn try_process(&self, body: &[Value]) -> Result<Self::Output> {
        body_is!(body, [path]);
        value_is!(path, Value::Variant(path));
        value_is!(&**path, Value::ObjectPath(path));

        Ok(path.to_string())
    }
}

impl SubscriptionResource for Resource {
    type Output = String;

    fn try_process(&self, path: &str, interface: &str, items: &[Value]) -> Result<Self::Output> {
        path_is!(path, "/org/freedesktop/NetworkManager");
        interface_is!(interface, "org.freedesktop.NetworkManager");

        for item in items {
            value_is!(item, Value::DictEntry(key, value));
            value_is!(&**key, Value::String(key));
            value_is!(&**value, Value::Variant(value));

            if key == "PrimaryConnection" {
                value_is!(&**value, Value::ObjectPath(value));
                return Ok(value.to_string());
            }
        }

        bail!("unrelated")
    }

    fn set_path(&mut self, _: String) {}
}
