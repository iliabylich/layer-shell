use crate::dbus::{
    DBus, Message, Oneshot, OneshotResource, Subscription, SubscriptionResource,
    messages::{body_is, interface_is, org_freedesktop_dbus::GetProperty, path_is, value_is},
    types::Value,
};
use anyhow::{Result, bail};

pub(crate) struct ActiveAccessPoint {
    oneshot: Oneshot<Resource>,
    subscription: Subscription<Resource>,
}

#[derive(Debug)]
pub(crate) enum ActiveAccessPointEvent {
    Connected(String),
    Disconnected,
}
impl From<String> for ActiveAccessPointEvent {
    fn from(path: String) -> Self {
        if path == "/" {
            Self::Disconnected
        } else {
            Self::Connected(path)
        }
    }
}

impl ActiveAccessPoint {
    pub(crate) fn new() -> Self {
        Self {
            oneshot: Oneshot::new(Resource),
            subscription: Subscription::new(Resource),
        }
    }

    pub(crate) fn reset(&mut self, dbus: &mut DBus) {
        self.subscription.reset(dbus);
        self.oneshot.reset();
    }

    pub(crate) fn init(&mut self, dbus: &mut DBus, path: &str) {
        self.subscription
            .start(dbus, "org.freedesktop.NetworkManager", path);
        self.oneshot.start(dbus, path.to_string());
    }

    pub(crate) fn on_message(&mut self, message: &Message) -> Option<ActiveAccessPointEvent> {
        None.or_else(|| self.oneshot.process(message))
            .or_else(|| self.subscription.process(message))
            .map(ActiveAccessPointEvent::from)
    }
}

struct Resource;

impl OneshotResource for Resource {
    type Input = String;
    type Output = String;

    fn make_request(&self, path: String) -> Message<'static> {
        GetProperty::new(
            "org.freedesktop.NetworkManager",
            path,
            "org.freedesktop.NetworkManager.Device.Wireless",
            "ActiveAccessPoint",
        )
        .into()
    }

    fn try_process(&self, body: &[Value]) -> Result<Self::Output> {
        body_is!(body, [active_access_point]);
        value_is!(active_access_point, Value::Variant(active_access_point));
        value_is!(
            &**active_access_point,
            Value::ObjectPath(active_access_point)
        );

        Ok(active_access_point.to_string())
    }
}

impl SubscriptionResource for Resource {
    type Output = String;

    fn try_process(&self, path: &str, interface: &str, items: &[Value]) -> Result<Self::Output> {
        path_is!(path, "/org/freedesktop/NetworkManager");
        interface_is!(interface, "org.freedesktop.NetworkManager.Device.Wireless");

        for item in items {
            value_is!(item, Value::DictEntry(key, value));
            value_is!(&**key, Value::String(key));
            value_is!(&**value, Value::Variant(value));

            if key == "ActiveAccessPoint" {
                value_is!(&**value, Value::ObjectPath(value));
                return Ok(value.to_string());
            }
        }

        bail!("unrelated")
    }

    fn set_path(&mut self, _: String) {}
}
