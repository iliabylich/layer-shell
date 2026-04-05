use crate::{
    dbus::types::{OutgoingMessage, Value},
    utils::StringRef,
};

pub(crate) struct AddMatch;

impl AddMatch {
    pub(crate) fn build(sender: StringRef, path: StringRef) -> OutgoingMessage {
        Self::build_from_rule(format!(
            "type='signal',sender='{sender}',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged',path='{path}'"
        ))
    }

    pub(crate) fn build_from_rule(rule: String) -> OutgoingMessage {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: StringRef::new("/org/freedesktop/DBus"),
            member: StringRef::new("AddMatch"),
            interface: Some(StringRef::new("org.freedesktop.DBus")),
            destination: Some(StringRef::new("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![Value::LongString(rule)],
        }
    }
}
