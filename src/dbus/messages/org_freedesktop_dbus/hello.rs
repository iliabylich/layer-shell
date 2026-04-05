use crate::{dbus::types::OutgoingMessage, utils::StringRef};

pub(crate) struct Hello;

impl From<Hello> for OutgoingMessage {
    fn from(_: Hello) -> OutgoingMessage {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: StringRef::new("/org/freedesktop/DBus"),
            member: StringRef::new("Hello"),
            interface: Some(StringRef::new("org.freedesktop.DBus")),
            destination: Some(StringRef::new("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![],
        }
    }
}
