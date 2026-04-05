use crate::{
    dbus::types::{OutgoingMessage, Value},
    utils::StringRef,
};

pub(crate) struct RemoveMatch;

impl RemoveMatch {
    pub(crate) fn build(path: StringRef) -> OutgoingMessage {
        Self::build_from_rule(format!(
            "type='signal',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged',path='{path}'"
        ))
    }

    pub(crate) fn build_from_rule(rule: String) -> OutgoingMessage {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: StringRef::new("/org/freedesktop/DBus"),
            member: StringRef::new("RemoveMatch"),
            interface: Some(StringRef::new("org.freedesktop.DBus")),
            destination: Some(StringRef::new("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![Value::LongString(rule)],
        }
    }
}
