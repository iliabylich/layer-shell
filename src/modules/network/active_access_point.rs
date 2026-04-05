use crate::{
    dbus::{
        MethodCall, Subscription,
        decoder::{IncomingMessage, Value},
        messages::{interface_is, org_freedesktop_dbus::GetProperty, path_is, value_is},
    },
    sansio::DBusConnectionKind,
    utils::StringRef,
};
use anyhow::{Context as _, bail};

pub(crate) struct ActiveAccessPoint {
    get: MethodCall<StringRef, StringRef, ()>,
    subscription: Subscription<StringRef>,
}

#[derive(Debug)]
pub(crate) enum ActiveAccessPointEvent {
    Connected(StringRef),
    Disconnected,
}
impl From<StringRef> for ActiveAccessPointEvent {
    fn from(path: StringRef) -> Self {
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
            get: GET,
            subscription: SUBSCRIPTION,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.subscription.reset();
        self.get.reset();
    }

    pub(crate) fn init(&mut self, path: StringRef) {
        self.subscription
            .start("org.freedesktop.NetworkManager", path.clone());
        self.get.send(path);
    }

    pub(crate) fn on_message(
        &mut self,
        message: IncomingMessage<'_>,
    ) -> Option<ActiveAccessPointEvent> {
        None.or_else(|| self.get.try_recv(message).ok().flatten())
            .or_else(|| self.subscription.process(message))
            .map(ActiveAccessPointEvent::from)
    }
}

const GET: MethodCall<StringRef, StringRef, ()> = MethodCall::builder()
    .send(&|path, _| {
        GetProperty::build(
            "org.freedesktop.NetworkManager",
            path,
            "org.freedesktop.NetworkManager.Device.Wireless",
            "ActiveAccessPoint",
        )
    })
    .try_process(&|mut body, _| {
        let active_access_point = body.try_next()?.context("no ActiveAccessPoint in Body")?;
        value_is!(active_access_point, Value::Variant(active_access_point));
        let active_access_point = active_access_point.materialize()?;
        value_is!(active_access_point, Value::ObjectPath(active_access_point));

        Ok(StringRef::new(active_access_point))
    })
    .kind(DBusConnectionKind::System);

const SUBSCRIPTION: Subscription<StringRef> = Subscription::builder()
    .try_process(&|mut body, path, subscribed_to| {
        path_is!(path, subscribed_to);

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
                return Ok(StringRef::new(value));
            }
        }

        bail!("unrelated")
    })
    .kind(DBusConnectionKind::System);
