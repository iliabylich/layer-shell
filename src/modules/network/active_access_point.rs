use crate::{
    dbus::{
        Message, Oneshot, OneshotResource, Subscription, SubscriptionResource,
        decoder::{Body, IncomingMessage, Value},
        messages::{interface_is, org_freedesktop_dbus::GetProperty, path_is, value_is},
    },
    sansio::DBusQueue,
};
use anyhow::{Context as _, Result, bail};

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
    pub(crate) fn new(queue: DBusQueue) -> Self {
        Self {
            oneshot: Oneshot::new(Resource, queue.copy()),
            subscription: Subscription::new(Resource, queue.copy()),
        }
    }

    pub(crate) fn reset(&mut self) {
        self.subscription.reset();
        self.oneshot.reset();
    }

    pub(crate) fn init(&mut self, path: &str) {
        self.subscription
            .start("org.freedesktop.NetworkManager", path);
        self.oneshot.start(path.to_string());
    }

    pub(crate) fn on_message(
        &mut self,
        message: IncomingMessage<'_>,
    ) -> Option<ActiveAccessPointEvent> {
        None.or_else(|| self.oneshot.process(message).ok().flatten())
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

    fn try_process(&self, mut body: Body<'_>) -> Result<Self::Output> {
        let active_access_point = body.try_next()?.context("no ActiveAccessPoint in Body")?;
        value_is!(active_access_point, Value::Variant(active_access_point));
        let active_access_point = active_access_point.materialize()?;
        value_is!(active_access_point, Value::ObjectPath(active_access_point));

        Ok(active_access_point.to_string())
    }
}

impl SubscriptionResource for Resource {
    type Output = String;

    fn try_process(&self, path: &str, mut body: Body<'_>) -> Result<Self::Output> {
        path_is!(path, "/org/freedesktop/NetworkManager");

        let interface = body.try_next()?.context("no Interface in Body")?;
        value_is!(interface, Value::String(interface));
        interface_is!(interface, "org.freedesktop.NetworkManager.Device.Wireless");

        let attributes = body.try_next()?.context("no Attributes in Body")?;
        value_is!(attributes, Value::Array(attributes));
        let mut iter = attributes.iter();
        while let Some(attribute) = iter.try_next()? {
            value_is!(attribute, Value::DictEntry(dict_entry));
            let (key, value) = dict_entry.key_value()?;
            value_is!(key, Value::String(key));
            value_is!(value, Value::Variant(value));

            if key == "ActiveAccessPoint" {
                let value = value.materialize()?;
                value_is!(value, Value::ObjectPath(value));
                return Ok(value.to_string());
            }
        }

        bail!("unrelated")
    }

    fn set_path(&mut self, _: String) {}
}
