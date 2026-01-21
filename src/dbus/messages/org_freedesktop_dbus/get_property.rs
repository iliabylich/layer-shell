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

impl<'a> From<GetProperty<'a>> for Message<'a> {
    fn from(value: GetProperty<'a>) -> Self {
        Message::MethodCall {
            serial: 0,
            path: Cow::Borrowed(value.path),
            member: Cow::Borrowed("Get"),
            interface: Some(Cow::Borrowed("org.freedesktop.DBus.Properties")),
            destination: Some(Cow::Borrowed(value.destination)),
            sender: None,
            unix_fds: None,
            body: vec![
                Value::String(Cow::Borrowed(value.interface)),
                Value::String(Cow::Borrowed(value.property)),
            ],
        }
    }
}
