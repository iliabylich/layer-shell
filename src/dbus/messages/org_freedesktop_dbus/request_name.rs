use crate::dbus::types::{Message, Value};
use std::borrow::Cow;

#[derive(Debug)]
pub(crate) struct RequestName {
    name: Cow<'static, str>,
}
impl RequestName {
    pub(crate) fn new(name: Cow<'static, str>) -> Self {
        Self { name }
    }
}
impl From<RequestName> for Message {
    fn from(value: RequestName) -> Message {
        Message::MethodCall {
            serial: 0,
            path: Cow::Borrowed("/org/freedesktop/DBus"),
            member: Cow::Borrowed("RequestName"),
            interface: Some(Cow::Borrowed("org.freedesktop.DBus")),
            destination: Some(Cow::Borrowed("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![Value::String(value.name.to_string()), Value::UInt32(7)],
        }
    }
}
