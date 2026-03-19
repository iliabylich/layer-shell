use crate::{
    dbus::{
        Oneshot, OneshotResource, OutgoingMessage, Subscription, SubscriptionResource,
        decoder::{Body, IncomingMessage, Value},
        messages::{interface_is, org_freedesktop_dbus::GetProperty, path_is, value_is},
    },
    ffi::ShortString,
    sansio::DBusQueue,
};
use anyhow::{Context, Result, bail};

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
    pub(crate) fn new(queue: DBusQueue) -> Self {
        Self {
            oneshot: Oneshot::new(Resource, queue.copy()),
            subscription: Subscription::new(Resource, queue.copy()),
        }
    }

    pub(crate) fn init(&mut self) {
        self.oneshot.start(());
        self.subscription.start(
            "org.freedesktop.NetworkManager",
            "/org/freedesktop/NetworkManager",
        );
    }

    pub(crate) fn on_message(
        &mut self,
        message: IncomingMessage<'_>,
    ) -> Option<PrimaryConnectionEvent> {
        None.or_else(|| self.oneshot.process(message).ok().flatten())
            .or_else(|| self.subscription.process(message))
            .map(PrimaryConnectionEvent::from)
    }
}

struct Resource;
impl OneshotResource for Resource {
    type Input = ();
    type Output = String;

    fn make_request(&self, _input: Self::Input) -> OutgoingMessage<'static> {
        GetProperty::new(
            ShortString::from("org.freedesktop.NetworkManager"),
            ShortString::from("/org/freedesktop/NetworkManager"),
            "org.freedesktop.NetworkManager",
            "PrimaryConnection",
        )
        .into()
    }

    fn try_process(&self, mut body: Body<'_>) -> Result<Self::Output> {
        let path = body.try_next()?.context("empty Body")?;
        value_is!(path, Value::Variant(path));
        let path = path.materialize()?;
        value_is!(path, Value::ObjectPath(path));
        Ok(path.to_string())
    }
}

impl SubscriptionResource for Resource {
    type Output = String;

    fn try_process(&self, path: &str, mut body: Body<'_>) -> Result<Self::Output> {
        path_is!(path, "/org/freedesktop/NetworkManager");

        let interface = body.try_next()?.context("empty Body")?;
        value_is!(interface, Value::String(interface));
        interface_is!(interface, "org.freedesktop.NetworkManager");

        let items = body.try_next()?.context("no items in Body")?;
        value_is!(items, Value::Array(items));
        let mut iter = items.iter();

        while let Some(item) = iter.try_next()? {
            value_is!(item, Value::DictEntry(dict_entry));
            let (key, value) = dict_entry.key_value()?;
            value_is!(key, Value::String(key));
            value_is!(value, Value::Variant(value));

            if key == "PrimaryConnection" {
                let value = value.materialize()?;
                value_is!(value, Value::ObjectPath(value));
                return Ok(value.to_string());
            }
        }

        bail!("unrelated")
    }

    fn set_path(&mut self, _: String) {}
}
