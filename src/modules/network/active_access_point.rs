use crate::dbus::{
    DBus, Message,
    messages::{
        body_is, interface_is, message_is,
        org_freedesktop_dbus::{AddMatch, GetProperty, RemoveMatch},
        path_is, type_is, value_is,
    },
    types::{CompleteType, Value},
};
use anyhow::{Result, bail, ensure};

pub(crate) struct ActiveAccessPoint {
    path: Option<String>,
    reply_serial: Option<u32>,
}

#[derive(Debug)]
pub(crate) enum ActiveAccessPointEvent {
    Connected(String),
    Disconnected,
}
impl From<&str> for ActiveAccessPointEvent {
    fn from(path: &str) -> Self {
        if path == "/" {
            Self::Disconnected
        } else {
            Self::Connected(path.to_string())
        }
    }
}

impl ActiveAccessPoint {
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
            "org.freedesktop.NetworkManager.Device.Wireless",
            "ActiveAccessPoint",
        )
        .into();
        dbus.enqueue(&mut message);
        self.reply_serial = Some(message.serial())
    }

    fn try_parse_reply<'a>(&self, message: &'a Message<'a>) -> Result<&'a str> {
        ensure!(message.reply_serial() == self.reply_serial);
        message_is!(message, Message::MethodReturn { body, .. });
        body_is!(body, [active_access_point]);
        value_is!(active_access_point, Value::Variant(active_access_point));
        value_is!(
            &**active_access_point,
            Value::ObjectPath(active_access_point)
        );

        Ok(active_access_point)
    }

    pub(crate) fn reset(&mut self, dbus: &mut DBus) {
        self.unsubscribe(dbus);
    }

    pub(crate) fn init(&mut self, dbus: &mut DBus, path: &str) {
        self.unsubscribe(dbus);
        self.subscribe(dbus, path);
        self.request(dbus, path);
    }

    pub(crate) fn on_message(&mut self, message: &Message) -> Option<ActiveAccessPointEvent> {
        if let Ok(device) = self.try_parse_reply(message) {
            return Some(ActiveAccessPointEvent::from(device));
        }

        if let Ok(device) = try_parse_signal(message) {
            return Some(ActiveAccessPointEvent::from(device));
        }

        None
    }
}

fn try_parse_signal<'a>(message: &'a Message<'a>) -> Result<&'a str> {
    message_is!(
        message,
        Message::Signal {
            path,
            interface,
            body,
            ..
        }
    );

    interface_is!(interface, "org.freedesktop.DBus.Properties");
    body_is!(
        body,
        [Value::String(interface), Value::Array(item_t, items), _]
    );
    type_is!(item_t, CompleteType::DictEntry(key_t, value_t));
    type_is!(&**key_t, CompleteType::String);
    type_is!(&**value_t, CompleteType::Variant);

    path_is!(path, "/org/freedesktop/NetworkManager");
    interface_is!(interface, "org.freedesktop.NetworkManager.Device.Wireless");

    for item in items {
        value_is!(item, Value::DictEntry(key, value));
        value_is!(&**key, Value::String(key));
        value_is!(&**value, Value::Variant(value));

        if key == "ActiveAccessPoint" {
            value_is!(&**value, Value::ObjectPath(active_access_point));
            return Ok(active_access_point.as_ref());
        }
    }

    bail!("unrelated")
}
