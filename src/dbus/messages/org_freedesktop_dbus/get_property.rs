use crate::dbus::types::{Message, Value};
use std::borrow::Cow;

#[derive(Debug)]
pub(crate) struct GetProperty<'a> {
    destination: &'a str,
    path: &'a str,
    interface: &'a str,
    property: &'a str,
}

impl<'a> GetProperty<'a> {
    pub(crate) fn new(
        destination: &'a str,
        path: &'a str,
        interface: &'a str,
        property: &'a str,
    ) -> Self {
        Self {
            destination,
            path,
            interface,
            property,
        }
    }
}

impl From<GetProperty<'_>> for Message {
    fn from(value: GetProperty) -> Message {
        Message::MethodCall {
            serial: 0,
            path: Cow::Owned(value.path.to_string()),
            member: Cow::Borrowed("Get"),
            interface: Some(Cow::Borrowed("org.freedesktop.DBus.Properties")),
            destination: Some(Cow::Owned(value.destination.to_string())),
            sender: None,
            unix_fds: None,
            body: vec![
                Value::String(value.interface.to_string()),
                Value::String(value.property.to_string()),
            ],
        }
    }
}
