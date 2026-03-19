use crate::{
    dbus::types::{OutgoingMessage, Value},
    ffi::ShortString,
};

pub(crate) struct RemoveMatch {
    path: ShortString,
}

impl RemoveMatch {
    pub(crate) fn new(path: ShortString) -> Self {
        Self { path }
    }
}

impl From<RemoveMatch> for OutgoingMessage {
    fn from(value: RemoveMatch) -> Self {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: ShortString::from("/org/freedesktop/DBus"),
            member: ShortString::from("RemoveMatch"),
            interface: Some(ShortString::from("org.freedesktop.DBus")),
            destination: Some(ShortString::from("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![Value::LongString(format!(
                "type='signal',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged',path='{}'",
                value.path
            ))],
        }
    }
}
