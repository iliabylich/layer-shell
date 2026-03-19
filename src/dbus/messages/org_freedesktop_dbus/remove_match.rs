use crate::{
    dbus::types::{OutgoingMessage, Value},
    ffi::ShortString,
};

pub(crate) struct RemoveMatch {
    path: ShortString,
}

impl RemoveMatch {
    pub(crate) const fn new(path: ShortString) -> Self {
        Self { path }
    }
}

impl From<RemoveMatch> for OutgoingMessage {
    fn from(value: RemoveMatch) -> Self {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: ShortString::new_const("/org/freedesktop/DBus"),
            member: ShortString::new_const("RemoveMatch"),
            interface: Some(ShortString::new_const("org.freedesktop.DBus")),
            destination: Some(ShortString::new_const("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![Value::LongString(format!(
                "type='signal',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged',path='{}'",
                value.path
            ))],
        }
    }
}
