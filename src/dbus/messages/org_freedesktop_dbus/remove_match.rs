use crate::{
    dbus::types::{OutgoingMessage, Value},
    utils::StringRef,
};

pub(crate) struct RemoveMatch {
    rule: String,
}

impl RemoveMatch {
    pub(crate) fn new(path: StringRef) -> Self {
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
            path: StringRef::new("/org/freedesktop/DBus"),
            member: StringRef::new("RemoveMatch"),
            interface: Some(StringRef::new("org.freedesktop.DBus")),
            destination: Some(StringRef::new("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![Value::LongString(value.rule)],
        }
    }
}
