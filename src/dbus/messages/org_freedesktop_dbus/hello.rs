use crate::{dbus::types::OutgoingMessage, ffi::ShortString};
use std::borrow::Cow;

pub(crate) struct Hello;

impl From<Hello> for OutgoingMessage<'static> {
    fn from(_: Hello) -> OutgoingMessage<'static> {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: Cow::Borrowed("/org/freedesktop/DBus"),
            member: Cow::Borrowed("Hello"),
            interface: Some(Cow::Borrowed("org.freedesktop.DBus")),
            destination: Some(ShortString::from("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![],
        }
    }
}
