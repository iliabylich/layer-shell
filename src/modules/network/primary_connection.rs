use crate::dbus::{
    DBus, Message,
    messages::{
        body_is, interface_is, message_is,
        org_freedesktop_dbus::{AddMatch, GetProperty, PropertiesChanged},
        path_is, value_is,
    },
    types::Value,
};
use anyhow::{Context as _, Result, ensure};

#[derive(Debug)]
pub(crate) struct PrimaryConnection {
    reply_serial: Option<u32>,
}

pub(crate) enum PrimaryConnectionEvent {
    Connected(String),
    Disconnected,
}

impl From<String> for PrimaryConnectionEvent {
    fn from(path: String) -> Self {
        if path == "/" {
            PrimaryConnectionEvent::Disconnected
        } else {
            PrimaryConnectionEvent::Connected(path)
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

    fn try_parse_reply(&self, message: &Message) -> Result<String> {
        ensure!(self.reply_serial == message.reply_serial());
        message_is!(message, Message::MethodReturn { body, .. });
        body_is!(body, [path]);
        value_is!(path, Value::Variant(path));
        value_is!(&**path, Value::ObjectPath(path));

        Ok(path.to_string())
    }

    fn try_parse_signal(&self, message: &Message) -> Result<String> {
        let PropertiesChanged {
            path,
            interface,
            changes,
        } = PropertiesChanged::try_from(message)?;

        path_is!(path, "/org/freedesktop/NetworkManager");
        interface_is!(interface, "org.freedesktop.NetworkManager");

        let path = changes.get("PrimaryConnection").context("unrelated")?;
        value_is!(path, Value::ObjectPath(path));

        Ok(path.to_string())
    }

    pub(crate) fn init(&mut self, dbus: &mut DBus) {
        self.request(dbus);
        self.subscribe(dbus);
    }

    pub(crate) fn on_message(&mut self, message: &Message) -> Option<PrimaryConnectionEvent> {
        if let Ok(path) = self.try_parse_reply(message) {
            return Some(PrimaryConnectionEvent::from(path));
        }

        if let Ok(path) = self.try_parse_signal(message) {
            return Some(PrimaryConnectionEvent::from(path));
        }

        None
    }
}
