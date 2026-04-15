use crate::{modules::SystemDBus, utils::StringRef};
use anyhow::Context as _;
use mini_sansio_dbus::{
    IncomingBody, IncomingMessage, IncomingValue, MethodCall, Subscription, interface_is,
    messages::org_freedesktop_dbus::GetProperty, path_is, value_is,
};

pub(crate) struct PrimaryDevice {
    get: MethodCall<StringRef, StringRef, ()>,
    subscription: Subscription<StringRef>,
}

#[derive(Debug)]
pub(crate) enum PrimaryDeviceEvent {
    Connected(StringRef),
    Disconnected,
}
impl From<StringRef> for PrimaryDeviceEvent {
    fn from(path: StringRef) -> Self {
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
        self.subscription.reset(SystemDBus::queue());
        self.get.reset();
    }

    pub(crate) fn init(&mut self, path: StringRef) {
        self.subscription.start(
            "org.freedesktop.NetworkManager",
            path.to_string(),
            SystemDBus::queue(),
        );
        self.get.send(path, SystemDBus::queue());
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

const GET: MethodCall<StringRef, StringRef, ()> = MethodCall::builder()
    .send(&|path: StringRef, _data| {
        GetProperty::build(
            "org.freedesktop.NetworkManager",
            path.as_str(),
            "org.freedesktop.NetworkManager.Connection.Active",
            "Devices",
        )
    })
    .try_process(&|mut body: IncomingBody<'_>, _data| {
        let devices = body.try_next()?.context("no Devices in Body")?;
        value_is!(devices, IncomingValue::Variant(devices));
        let devices = devices.materialize()?;
        value_is!(devices, IncomingValue::Array(devices));
        let mut iter = devices.iter();
        let device = iter.try_next()?.context("expected at least one device")?;
        value_is!(device, IncomingValue::ObjectPath(device));

        Ok(StringRef::new(device))
    });

const SUBSCRIPTION: Subscription<StringRef> =
    Subscription::new(&|mut body, path, subscribed_to| {
        path_is!(path, subscribed_to);

        let interface = body.try_next()?.context("no Interface in Body")?;
        value_is!(interface, IncomingValue::String(interface));
        interface_is!(
            interface,
            "org.freedesktop.NetworkManager.Connection.Active"
        );

        let items = body.try_next()?.context("no Items in Body")?;
        value_is!(items, IncomingValue::Array(items));
        let mut iter = items.iter();
        while let Some(item) = iter.try_next()? {
            value_is!(item, IncomingValue::DictEntry(dict_entry));
            let (key, value) = dict_entry.key_value()?;
            value_is!(key, IncomingValue::String(key));
            value_is!(value, IncomingValue::Variant(value));

            if key == "Devices" {
                let devices = value;
                let devices = devices.materialize()?;
                value_is!(devices, IncomingValue::Array(devices));
                let mut iter = devices.iter();
                let device = iter.try_next()?.context("expected at least one device")?;
                value_is!(device, IncomingValue::ObjectPath(device));
                return Ok(StringRef::new(device));
            }
        }

        Err(anyhow::anyhow!("unrelated").into())
    });
