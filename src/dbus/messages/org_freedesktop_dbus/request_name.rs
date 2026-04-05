use crate::{
    dbus::types::{OutgoingMessage, Value},
    utils::StringRef,
};

pub(crate) struct RequestName;

impl RequestName {
    pub(crate) fn build(name: impl Into<StringRef>) -> OutgoingMessage {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: StringRef::new("/org/freedesktop/DBus"),
            member: StringRef::new("RequestName"),
            interface: Some(StringRef::new("org.freedesktop.DBus")),
            destination: Some(StringRef::new("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![Value::StringRef(name.into()), Value::UInt32(7)],
        }
    }
}
