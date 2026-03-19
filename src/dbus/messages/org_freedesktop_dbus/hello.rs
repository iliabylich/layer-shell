use crate::{dbus::types::OutgoingMessage, ffi::ShortString};

pub(crate) struct Hello;

impl From<Hello> for OutgoingMessage<'static> {
    fn from(_: Hello) -> OutgoingMessage<'static> {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: ShortString::from("/org/freedesktop/DBus"),
            member: ShortString::from("Hello"),
            interface: Some(ShortString::from("org.freedesktop.DBus")),
            destination: Some(ShortString::from("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![],
        }
    }
}
