use crate::dbus::{
    DBus, Message,
    messages::{
        body_is, interface_is, message_is,
        org_freedesktop_dbus::{AddMatch, GetProperty, PropertiesChanged, RemoveMatch},
        path_is, value_is,
    },
    types::{CompleteType, Value},
};
use anyhow::{Context, Result, ensure};

pub(crate) struct PrimaryDevice {
    path: Option<String>,
    reply_serial: Option<u32>,
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
            path: None,
            reply_serial: None,
        }
    }

    fn unsubscribe(&mut self, dbus: &mut DBus) {
        let Some(old_path) = self.path.take() else {
            return;
        };

        let mut message: Message = RemoveMatch::new(&old_path).into();
        dbus.enqueue(&mut message);
    }

    fn subscribe(&mut self, dbus: &mut DBus, path: &str) {
        let mut message: Message = AddMatch::new(path).into();
        dbus.enqueue(&mut message);
        self.path = Some(path.to_string())
    }

    fn request(&mut self, dbus: &mut DBus, path: &str) {
        let mut message: Message = GetProperty::new(
            "org.freedesktop.NetworkManager",
            path,
            "org.freedesktop.NetworkManager.Connection.Active",
            "Devices",
        )
        .into();
        dbus.enqueue(&mut message);
        self.reply_serial = Some(message.serial());
    }

    fn try_parse_reply(&self, message: &Message) -> Result<String> {
        ensure!(message.reply_serial() == self.reply_serial);
        message_is!(message, Message::MethodReturn { body, .. });
        body_is!(body, [devices]);
        value_is!(devices, Value::Variant(devices));
        value_is!(&**devices, Value::Array(CompleteType::ObjectPath, devices));
        let device = sole(devices)?;
        value_is!(device, Value::ObjectPath(device));

        Ok(device.to_string())
    }

    fn try_parse_signal(&self, message: &Message) -> Result<String> {
        let PropertiesChanged {
            path,
            interface,
            changes,
        } = PropertiesChanged::try_from(message)?;

        path_is!(path, "/org/freedesktop/NetworkManager");
        interface_is!(
            interface,
            "org.freedesktop.NetworkManager.Connection.Active"
        );

        let devices = changes.get("Devices").context("unrelated")?;
        value_is!(devices, Value::Array(CompleteType::ObjectPath, devices));
        let device = sole(devices)?;
        value_is!(device, Value::ObjectPath(device));

        Ok(device.to_string())
    }

    pub(crate) fn reset(&mut self, dbus: &mut DBus) {
        self.unsubscribe(dbus);
    }

    pub(crate) fn init(&mut self, path: String, dbus: &mut DBus) {
        self.unsubscribe(dbus);
        self.subscribe(dbus, &path);
        self.request(dbus, &path);
    }

    pub(crate) fn on_message(&mut self, message: &Message) -> Option<PrimaryDeviceEvent> {
        if let Ok(device) = self.try_parse_reply(message) {
            return Some(PrimaryDeviceEvent::from(device));
        }

        if let Ok(device) = self.try_parse_signal(message) {
            return Some(PrimaryDeviceEvent::from(device));
        }

        None
    }
}

fn sole(devices: &[Value]) -> Result<&Value> {
    let mut iter = devices.iter();
    let device = iter.next().context("empty device list")?;
    ensure!(iter.next().is_none(), "1+ devices");
    Ok(device)
}
