use crate::dbus::types::{Message, Value};
use std::borrow::Cow;

pub(crate) struct AddMatch<'a> {
    sender: &'a str,
    path: &'a str,
}

impl<'a> AddMatch<'a> {
    pub(crate) fn new(sender: &'a str, path: &'a str) -> Self {
        Self { sender, path }
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
            body: vec![Value::String(Cow::Owned(format!(
                "type='signal',sender='{}',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged',path='{}'",
                value.sender, value.path
            )))],
        }
    }
}
