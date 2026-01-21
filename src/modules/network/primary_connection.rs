use crate::dbus::{
    DBus, Message,
    messages::{
        body_is, interface_is, message_is,
        org_freedesktop_dbus::{AddMatch, GetProperty},
        path_is, type_is, value_is,
    },
    types::{CompleteType, Value},
};
use anyhow::{Result, bail, ensure};

#[derive(Debug)]
pub(crate) struct PrimaryConnection {
    reply_serial: Option<u32>,
}

pub(crate) enum PrimaryConnectionEvent {
    Connected(String),
    Disconnected,
}

impl From<&str> for PrimaryConnectionEvent {
    fn from(path: &str) -> Self {
        if path == "/" {
            PrimaryConnectionEvent::Disconnected
        } else {
            PrimaryConnectionEvent::Connected(path.to_string())
        }
    }
}

impl PrimaryConnection {
    pub(crate) fn new() -> Self {
        Self { reply_serial: None }
    }

    fn request(&mut self, dbus: &mut DBus) {
        let mut message: Message = GetProperty::new(
            "org.freedesktop.NetworkManager",
            "/org/freedesktop/NetworkManager",
            "org.freedesktop.NetworkManager",
            "PrimaryConnection",
        )
        .into();
        dbus.enqueue(&mut message);
        self.reply_serial = Some(message.serial());
    }

    fn subscribe(&self, dbus: &mut DBus) {
        let mut message: Message = AddMatch::new("/org/freedesktop/NetworkManager").into();
        dbus.enqueue(&mut message);
    }

    fn try_parse_reply<'a>(&self, message: &'a Message<'a>) -> Result<&'a str> {
        ensure!(self.reply_serial == message.reply_serial());
        message_is!(message, Message::MethodReturn { body, .. });
        body_is!(body, [path]);
        value_is!(path, Value::Variant(path));
        value_is!(&**path, Value::ObjectPath(path));

        Ok(path)
    }

    pub(crate) fn init(&mut self, dbus: &mut DBus) {
        self.request(dbus);
        self.subscribe(dbus);
    }

    pub(crate) fn on_message(&mut self, message: &Message) -> Option<PrimaryConnectionEvent> {
        if let Ok(path) = self.try_parse_reply(message) {
            return Some(PrimaryConnectionEvent::from(path));
        }

        if let Ok(path) = try_parse_signal(message) {
            return Some(PrimaryConnectionEvent::from(path));
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
    interface_is!(interface, "org.freedesktop.NetworkManager");

    for item in items {
        value_is!(item, Value::DictEntry(key, value));
        value_is!(&**key, Value::String(key));
        value_is!(&**value, Value::Variant(value));

        if key == "PrimaryConnection" {
            value_is!(&**value, Value::ObjectPath(path));
            return Ok(path.as_ref());
        }
    }

    bail!("unrelated")
}
