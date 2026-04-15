use crate::{modules::SystemDBus, utils::StringRef};
use anyhow::Context as _;
use mini_sansio_dbus::{
    IncomingMessage, IncomingValue, MethodCall, Subscription, interface_is,
    messages::org_freedesktop_dbus::GetProperty, path_is, value_is,
};

pub(crate) struct PrimaryConnection {
    get: MethodCall<(), StringRef, ()>,
    subscription: Subscription<StringRef>,
}

pub(crate) enum PrimaryConnectionEvent {
    Connected(StringRef),
    Disconnected,
}

impl From<StringRef> for PrimaryConnectionEvent {
    fn from(path: StringRef) -> Self {
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
            get: GET,
            subscription: SUBSCRIPTION,
        }
    }

    pub(crate) fn init(&mut self) {
        self.get.send((), SystemDBus::queue());
        self.subscription.start(
            "org.freedesktop.NetworkManager",
            "/org/freedesktop/NetworkManager",
            SystemDBus::queue(),
        );
    }

    pub(crate) fn on_message(
        &mut self,
        message: IncomingMessage<'_>,
    ) -> Option<PrimaryConnectionEvent> {
        None.or_else(|| self.get.try_recv(message).ok().flatten())
            .or_else(|| self.subscription.process(message))
            .map(PrimaryConnectionEvent::from)
    }
}

const GET: MethodCall<(), StringRef, ()> = MethodCall::builder()
    .send(&|_input, _data| {
        GetProperty::build(
            "org.freedesktop.NetworkManager",
            "/org/freedesktop/NetworkManager",
            "org.freedesktop.NetworkManager",
            "PrimaryConnection",
        )
    })
    .try_process(&|mut body, _data| {
        let path = body.try_next()?.context("empty Body")?;
        value_is!(path, IncomingValue::Variant(path));
        let path = path.materialize()?;
        value_is!(path, IncomingValue::ObjectPath(path));
        Ok(StringRef::new(path))
    });

const SUBSCRIPTION: Subscription<StringRef> =
    Subscription::new(&|mut body, path, _subscribed_to| {
        path_is!(path, "/org/freedesktop/NetworkManager");

        let interface = body.try_next()?.context("empty Body")?;
        value_is!(interface, IncomingValue::String(interface));
        interface_is!(interface, "org.freedesktop.NetworkManager");

        let items = body.try_next()?.context("no items in Body")?;
        value_is!(items, IncomingValue::Array(items));
        let mut iter = items.iter();

        while let Some(item) = iter.try_next()? {
            value_is!(item, IncomingValue::DictEntry(dict_entry));
            let (key, value) = dict_entry.key_value()?;
            value_is!(key, IncomingValue::String(key));
            value_is!(value, IncomingValue::Variant(value));

            if key == "PrimaryConnection" {
                let value = value.materialize()?;
                value_is!(value, IncomingValue::ObjectPath(value));
                return Ok(StringRef::new(value));
            }
        }

        Err(anyhow::anyhow!("unrelated").into())
    });
