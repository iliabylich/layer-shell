use crate::{
    dbus::types::{OutgoingMessage, Value},
    ffi::ShortString,
};

pub(crate) struct RemoveMatch {
    rule: String,
}

impl RemoveMatch {
    pub(crate) fn new(path: ShortString) -> Self {
        Self {
            rule: format!(
                "type='signal',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged',path='{path}'"
            ),
        }
    }

    pub(crate) fn from_rule(rule: String) -> Self {
        Self { rule }
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
            body: vec![Value::LongString(value.rule)],
        }
    }
}
