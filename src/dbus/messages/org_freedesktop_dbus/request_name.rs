use crate::{
    dbus::types::{OutgoingMessage, Value},
    ffi::ShortString,
};
use std::borrow::Cow;

#[derive(Debug)]
pub(crate) struct RequestName<'a> {
    name: &'a str,
}
impl<'a> RequestName<'a> {
    pub(crate) fn new(name: &'a str) -> Self {
        Self { name }
    }
}
impl<'a> From<RequestName<'a>> for OutgoingMessage<'a> {
    fn from(value: RequestName) -> OutgoingMessage {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: ShortString::from("/org/freedesktop/DBus"),
            member: ShortString::from("RequestName"),
            interface: Some(ShortString::from("org.freedesktop.DBus")),
            destination: Some(ShortString::from("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![Value::String(Cow::Borrowed(value.name)), Value::UInt32(7)],
        }
    }
}
