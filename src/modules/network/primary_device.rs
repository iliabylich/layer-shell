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

pub(crate) struct PrimaryDevice {
    oneshot: Oneshot<Resource>,
    subscription: Subscription<Resource>,
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
    ) -> Option<PrimaryDeviceEvent> {
        None.or_else(|| self.oneshot.try_rev(message).ok().flatten())
            .or_else(|| self.subscription.process(message))
            .map(PrimaryDeviceEvent::from)
    }
}

struct Resource;

impl OneshotResource for Resource {
    type Input = ShortString;
    type Output = ShortString;

    fn request(&self, path: ShortString) -> impl Into<OutgoingMessage> {
        GetProperty::new(
            ShortString::new_const("org.freedesktop.NetworkManager"),
            path,
            ShortString::new_const("org.freedesktop.NetworkManager.Connection.Active"),
            ShortString::new_const("Devices"),
        )
    }

    fn try_recv(&self, mut body: Body<'_>) -> Result<Self::Output> {
        let devices = body.try_next()?.context("no Devices in Body")?;
        value_is!(devices, Value::Variant(devices));
        let devices = devices.materialize()?;
        value_is!(devices, Value::Array(devices));
        let mut iter = devices.iter();
        let device = iter.try_next()?.context("expected at least one device")?;
        value_is!(device, Value::ObjectPath(device));

        Ok(ShortString::from(device))
    }
}

impl SubscriptionResource for Resource {
    type Output = ShortString;

    fn try_process(&self, path: ShortString, mut body: Body<'_>) -> Result<Self::Output> {
        path_is!(path, "/org/freedesktop/NetworkManager");

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
    }

    fn set_path(&mut self, _: ShortString) {}
}
