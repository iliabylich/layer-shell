use crate::dbus::types::{Message, Value};
use std::borrow::Cow;

pub(crate) struct AddMatch<'a> {
    path: &'a str,
}

impl<'a> AddMatch<'a> {
    pub(crate) fn new(path: &'a str) -> Self {
        Self { path }
    }
}

impl<'a> From<AddMatch<'a>> for Message<'a> {
    fn from(value: AddMatch) -> Message {
        Message::MethodCall {
            serial: 0,
            path: Cow::Borrowed("/org/freedesktop/DBus"),
            member: Cow::Borrowed("AddMatch"),
            interface: Some(Cow::Borrowed("org.freedesktop.DBus")),
            destination: Some(Cow::Borrowed("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![Value::String(format!(
                "type='signal',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged',path='{}'",
                value.path
            ))],
        }
    }
}
