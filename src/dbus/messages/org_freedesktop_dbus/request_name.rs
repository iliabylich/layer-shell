use crate::{
    dbus::types::{OutgoingMessage, Value},
    ffi::ShortString,
};

#[derive(Debug)]
pub(crate) struct RequestName {
    name: ShortString,
}

impl RequestName {
    pub(crate) const fn new(name: ShortString) -> Self {
        Self { name }
    }
}

impl From<RequestName> for OutgoingMessage {
    fn from(value: RequestName) -> OutgoingMessage {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: ShortString::new_const("/org/freedesktop/DBus"),
            member: ShortString::new_const("RequestName"),
            interface: Some(ShortString::new_const("org.freedesktop.DBus")),
            destination: Some(ShortString::new_const("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![Value::ShortString(value.name), Value::UInt32(7)],
        }
    }
}
