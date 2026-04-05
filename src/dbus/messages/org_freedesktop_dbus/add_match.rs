use crate::{
    dbus::types::{OutgoingMessage, Value},
    utils::StringRef,
};

pub(crate) struct AddMatch {
    rule: String,
}

impl AddMatch {
    pub(crate) fn new(sender: StringRef, path: StringRef) -> Self {
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
            path: StringRef::new("/org/freedesktop/DBus"),
            member: StringRef::new("AddMatch"),
            interface: Some(StringRef::new("org.freedesktop.DBus")),
            destination: Some(StringRef::new("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![Value::LongString(value.rule)],
        }
    }
}
