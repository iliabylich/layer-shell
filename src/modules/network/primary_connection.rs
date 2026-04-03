use crate::{
    dbus::{
        MethodCall, Subscription,
        decoder::{IncomingMessage, Value},
        messages::{interface_is, org_freedesktop_dbus::GetProperty, path_is, value_is},
    },
    ffi::ShortString,
    sansio::DBusConnectionKind,
};
use anyhow::{Context as _, bail};

pub(crate) struct PrimaryConnection {
    get: MethodCall<(), ShortString, ()>,
    subscription: Subscription<ShortString>,
}

pub(crate) enum PrimaryConnectionEvent {
    Connected(ShortString),
    Disconnected,
}

impl From<ShortString> for PrimaryConnectionEvent {
    fn from(path: ShortString) -> Self {
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
        self.get.send(());
        self.subscription.start(
            ShortString::new_const("org.freedesktop.NetworkManager"),
            ShortString::new_const("/org/freedesktop/NetworkManager"),
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

const GET: MethodCall<(), ShortString, ()> = MethodCall::builder()
    .send(&|_input, _data| {
        GetProperty::new(
            ShortString::new_const("org.freedesktop.NetworkManager"),
            ShortString::new_const("/org/freedesktop/NetworkManager"),
            ShortString::new_const("org.freedesktop.NetworkManager"),
            ShortString::new_const("PrimaryConnection"),
        )
        .into()
    })
    .try_process(&|mut body, _data| {
        let path = body.try_next()?.context("empty Body")?;
        value_is!(path, Value::Variant(path));
        let path = path.materialize()?;
        value_is!(path, Value::ObjectPath(path));
        Ok(ShortString::from(path))
    })
    .kind(DBusConnectionKind::System);

const SUBSCRIPTION: Subscription<ShortString> = Subscription::builder()
    .try_process(&|mut body, path, _subscribed_to| {
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
                return Ok(ShortString::from(value));
            }
        }

        bail!("unrelated")
    })
    .kind(DBusConnectionKind::System);
