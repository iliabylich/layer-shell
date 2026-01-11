use crate::dbus::{Message, types::Value};
use std::borrow::Cow;

pub(crate) struct SetProperty<'a> {
    destination: &'a str,
    path: &'a str,
    interface: &'a str,
    property: &'a str,
    value: Value,
}

impl<'a> SetProperty<'a> {
    pub(crate) fn new(
        destination: &'a str,
        path: &'a str,
        interface: &'a str,
        property: &'a str,
        value: Value,
    ) -> Self {
        Self {
            destination,
            path,
            interface,
            property,
            value,
        }
    }
}

impl From<SetProperty<'_>> for Message {
    fn from(message: SetProperty<'_>) -> Self {
        Message::MethodCall {
            serial: 0,
            path: Cow::Owned(message.path.to_string()),
            member: Cow::Borrowed("Set"),
            interface: Some(Cow::Borrowed("org.freedesktop.DBus.Properties")),
            destination: Some(Cow::Owned(message.destination.to_string())),
            sender: None,
            unix_fds: None,
            body: vec![
                Value::String(message.interface.to_string()),
                Value::String(message.property.to_string()),
                Value::Variant(Box::new(message.value)),
            ],
        }
    }
}
