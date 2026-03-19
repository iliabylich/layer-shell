use crate::{
    dbus::types::{OutgoingMessage, Value},
    ffi::ShortString,
};

pub(crate) struct AddMatch {
    sender: ShortString,
    path: ShortString,
}

impl AddMatch {
    pub(crate) fn new(sender: ShortString, path: ShortString) -> Self {
        Self { sender, path }
    }
}

impl From<AddMatch> for OutgoingMessage {
    fn from(value: AddMatch) -> OutgoingMessage {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: ShortString::from("/org/freedesktop/DBus"),
            member: ShortString::from("AddMatch"),
            interface: Some(ShortString::from("org.freedesktop.DBus")),
            destination: Some(ShortString::from("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![Value::LongString(format!(
                "type='signal',sender='{}',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged',path='{}'",
                value.sender, value.path
            ))],
        }
    }
}
