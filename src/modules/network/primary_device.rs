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

pub(crate) struct PrimaryDevice {
    get: MethodCall<ShortString, ShortString, ()>,
    subscription: Subscription<ShortString>,
}

#[derive(Debug)]
pub(crate) enum PrimaryDeviceEvent {
    Connected(ShortString),
    Disconnected,
}
impl From<ShortString> for PrimaryDeviceEvent {
    fn from(path: ShortString) -> Self {
        if path == "/" {
            Self::Disconnected
        } else {
            Self::Connected(path)
        }
    }
}

impl PrimaryDevice {
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

    pub(crate) fn init(&mut self, path: ShortString) {
        self.subscription.start(
            ShortString::new_const("org.freedesktop.NetworkManager"),
            path,
        );
        self.get.send(path);
    }

    pub(crate) fn on_message(
        &mut self,
        message: IncomingMessage<'_>,
    ) -> Option<PrimaryDeviceEvent> {
        None.or_else(|| self.get.try_recv(message).ok().flatten())
            .or_else(|| self.subscription.process(message))
            .map(PrimaryDeviceEvent::from)
    }
}

const GET: MethodCall<ShortString, ShortString, ()> = MethodCall::builder()
    .send(&|path, _data| {
        GetProperty::new(
            ShortString::new_const("org.freedesktop.NetworkManager"),
            path,
            ShortString::new_const("org.freedesktop.NetworkManager.Connection.Active"),
            ShortString::new_const("Devices"),
        )
        .into()
    })
    .try_process(&|mut body, _data| {
        let devices = body.try_next()?.context("no Devices in Body")?;
        value_is!(devices, Value::Variant(devices));
        let devices = devices.materialize()?;
        value_is!(devices, Value::Array(devices));
        let mut iter = devices.iter();
        let device = iter.try_next()?.context("expected at least one device")?;
        value_is!(device, Value::ObjectPath(device));

        Ok(ShortString::from(device))
    })
    .kind(DBusConnectionKind::System);

const SUBSCRIPTION: Subscription<ShortString> = Subscription::builder()
    .try_process(&|mut body, path, subscribed_to| {
        path_is!(path, subscribed_to);

        let interface = body.try_next()?.context("no Interface in Body")?;
        value_is!(interface, Value::String(interface));
        interface_is!(
            interface,
            "org.freedesktop.NetworkManager.Connection.Active"
        );

        let items = body.try_next()?.context("no Items in Body")?;
        value_is!(items, Value::Array(items));
        let mut iter = items.iter();
        while let Some(item) = iter.try_next()? {
            value_is!(item, Value::DictEntry(dict_entry));
            let (key, value) = dict_entry.key_value()?;
            value_is!(key, Value::String(key));
            value_is!(value, Value::Variant(value));

            if key == "Devices" {
                let devices = value;
                let devices = devices.materialize()?;
                value_is!(devices, Value::Array(devices));
                let mut iter = devices.iter();
                let device = iter.try_next()?.context("expected at least one device")?;
                value_is!(device, Value::ObjectPath(device));
                return Ok(ShortString::from(device));
            }
        }

        bail!("unrelated")
    })
    .kind(DBusConnectionKind::System);
