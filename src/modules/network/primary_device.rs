use crate::dbus::{
    DBus, Message, Oneshot, OneshotResource, Subscription, SubscriptionResource,
    messages::{body_is, interface_is, org_freedesktop_dbus::GetProperty, path_is, value_is},
    types::{CompleteType, Value},
};
use anyhow::{Context, Result, bail};

pub(crate) struct PrimaryDevice {
    oneshot: Oneshot<Resource>,
    subscription: Subscription<Resource>,
}

#[derive(Debug)]
pub(crate) enum PrimaryDeviceEvent {
    Connected(String),
    Disconnected,
}
impl From<String> for PrimaryDeviceEvent {
    fn from(path: String) -> Self {
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
            oneshot: Oneshot::new(Resource),
            subscription: Subscription::new(Resource),
        }
    }

    pub(crate) fn reset(&mut self, dbus: &mut DBus) -> Result<()> {
        self.subscription.reset(dbus)?;
        self.oneshot.reset();
        Ok(())
    }

    pub(crate) fn init(&mut self, path: String, dbus: &mut DBus) -> Result<()> {
        self.subscription.start(dbus, &path)?;
        self.oneshot.start(dbus, path)?;
        Ok(())
    }

    pub(crate) fn on_message(&mut self, message: &Message) -> Option<PrimaryDeviceEvent> {
        None.or_else(|| self.oneshot.process(message))
            .or_else(|| self.subscription.process(message))
            .map(PrimaryDeviceEvent::from)
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
            "org.freedesktop.NetworkManager.Connection.Active",
            "Devices",
        )
        .into()
    }

    fn try_process(&self, body: &[Value]) -> Result<Self::Output> {
        body_is!(body, [devices]);
        value_is!(devices, Value::Variant(devices));
        value_is!(&**devices, Value::Array(CompleteType::ObjectPath, devices));
        let device = devices.first().context("expected at least one device")?;
        value_is!(device, Value::ObjectPath(device));

        Ok(device.to_string())
    }
}

impl SubscriptionResource for Resource {
    type Output = String;

    fn try_process(&self, path: &str, interface: &str, items: &[Value]) -> Result<Self::Output> {
        path_is!(path, "/org/freedesktop/NetworkManager");
        interface_is!(
            interface,
            "org.freedesktop.NetworkManager.Connection.Active"
        );

        for item in items {
            value_is!(item, Value::DictEntry(key, value));
            value_is!(&**key, Value::String(key));
            value_is!(&**value, Value::Variant(value));

            if key == "Devices" {
                let devices = value;
                value_is!(&**devices, Value::Array(CompleteType::ObjectPath, devices));
                let device = devices.first().context("expected at least one device")?;
                value_is!(device, Value::ObjectPath(device));
                return Ok(device.to_string());
            }
        }

        bail!("unrelated")
    }

    fn set_path(&mut self, _: String) {}
}
