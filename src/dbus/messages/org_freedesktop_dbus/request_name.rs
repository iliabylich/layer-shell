use crate::{
    dbus::types::{OutgoingMessage, Value},
    utils::StringRef,
};

#[derive(Debug)]
pub(crate) struct RequestName {
    name: StringRef,
}

impl RequestName {
    pub(crate) const fn new(name: StringRef) -> Self {
        Self { name }
    }
}

impl From<RequestName> for OutgoingMessage {
    fn from(value: RequestName) -> OutgoingMessage {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: StringRef::new("/org/freedesktop/DBus"),
            member: StringRef::new("RequestName"),
            interface: Some(StringRef::new("org.freedesktop.DBus")),
            destination: Some(StringRef::new("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![Value::StringRef(value.name), Value::UInt32(7)],
        }
    }
}
