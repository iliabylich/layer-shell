use crate::{
    dbus::types::{OutgoingMessage, Value},
    ffi::ShortString,
};

pub(crate) struct AddMatch {
    rule: String,
}

impl AddMatch {
    pub(crate) fn new(sender: ShortString, path: ShortString) -> Self {
        Self {
            rule: format!(
                "type='signal',sender='{sender}',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged',path='{path}'"
            ),
        }
    }

    pub(crate) fn from_rule(rule: String) -> Self {
        Self { rule }
    }
}

impl From<AddMatch> for OutgoingMessage {
    fn from(value: AddMatch) -> OutgoingMessage {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: ShortString::new_const("/org/freedesktop/DBus"),
            member: ShortString::new_const("AddMatch"),
            interface: Some(ShortString::new_const("org.freedesktop.DBus")),
            destination: Some(ShortString::new_const("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![Value::LongString(value.rule)],
        }
    }
}
