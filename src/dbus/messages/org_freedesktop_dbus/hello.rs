use crate::{dbus::types::OutgoingMessage, ffi::ShortString};

pub(crate) struct Hello;

impl From<Hello> for OutgoingMessage {
    fn from(_: Hello) -> OutgoingMessage {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: ShortString::new_const("/org/freedesktop/DBus"),
            member: ShortString::new_const("Hello"),
            interface: Some(ShortString::new_const("org.freedesktop.DBus")),
            destination: Some(ShortString::new_const("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![],
        }
    }
}
