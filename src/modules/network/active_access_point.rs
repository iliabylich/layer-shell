use crate::{
    dbus::{
        OneshotMethodCall, Subscription, SubscriptionResource,
        decoder::{Body, IncomingMessage, Value},
        messages::{interface_is, org_freedesktop_dbus::GetProperty, path_is, value_is},
    },
    ffi::ShortString,
    sansio::DBusConnectionKind,
};
use anyhow::{Context as _, Result, bail};

pub(crate) struct ActiveAccessPoint {
    oneshot: OneshotMethodCall<ShortString, ShortString, ()>,
    subscription: Subscription<Resource>,
}

#[derive(Debug)]
pub(crate) enum ActiveAccessPointEvent {
    Connected(ShortString),
    Disconnected,
}
impl From<ShortString> for ActiveAccessPointEvent {
    fn from(path: ShortString) -> Self {
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
            oneshot: GET,
            subscription: Subscription::new(Resource::default(), DBusConnectionKind::System),
        }
    }

    pub(crate) fn reset(&mut self) {
        self.subscription.reset();
        self.oneshot.reset();
    }

    pub(crate) fn init(&mut self, path: ShortString) {
        self.subscription.start(
            ShortString::new_const("org.freedesktop.NetworkManager"),
            path,
        );
        self.oneshot.send(path);
    }

    pub(crate) fn on_message(
        &mut self,
        message: IncomingMessage<'_>,
    ) -> Option<ActiveAccessPointEvent> {
        None.or_else(|| self.oneshot.try_recv(message).ok().flatten())
            .or_else(|| self.subscription.process(message))
            .map(ActiveAccessPointEvent::from)
    }
}

const GET: OneshotMethodCall<ShortString, ShortString, ()> = OneshotMethodCall::builder()
    .send(&|path, _| {
        GetProperty::new(
            ShortString::new_const("org.freedesktop.NetworkManager"),
            path,
            ShortString::new_const("org.freedesktop.NetworkManager.Device.Wireless"),
            ShortString::new_const("ActiveAccessPoint"),
        )
        .into()
    })
    .try_process(&|mut body, _| {
        let active_access_point = body.try_next()?.context("no ActiveAccessPoint in Body")?;
        value_is!(active_access_point, Value::Variant(active_access_point));
        let active_access_point = active_access_point.materialize()?;
        value_is!(active_access_point, Value::ObjectPath(active_access_point));

        Ok(ShortString::from(active_access_point))
    })
    .kind(DBusConnectionKind::System);

#[derive(Default)]
struct Resource {
    path: Option<ShortString>,
}

impl SubscriptionResource for Resource {
    type Output = ShortString;

    fn try_process(&self, path: ShortString, mut body: Body<'_>) -> Result<Self::Output> {
        path_is!(path, self.path.context("no path")?);

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
                return Ok(ShortString::from(value));
            }
        }

        bail!("unrelated")
    }

    fn set_path(&mut self, path: ShortString) {
        self.path = Some(path)
    }
}
