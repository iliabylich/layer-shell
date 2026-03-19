use crate::{
    dbus::types::{OutgoingMessage, Value},
    ffi::ShortString,
};
use std::borrow::Cow;

pub(crate) struct RemoveMatch<'a> {
    path: &'a str,
}

impl<'a> RemoveMatch<'a> {
    pub(crate) fn new(path: &'a str) -> Self {
        Self { path }
    }
}

impl<'a> From<RemoveMatch<'a>> for OutgoingMessage<'a> {
    fn from(value: RemoveMatch) -> Self {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: ShortString::from("/org/freedesktop/DBus"),
            member: ShortString::from("RemoveMatch"),
            interface: Some(ShortString::from("org.freedesktop.DBus")),
            destination: Some(ShortString::from("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![Value::String(Cow::Owned(format!(
                "type='signal',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged',path='{}'",
                value.path
            )))],
        }
    }
}
