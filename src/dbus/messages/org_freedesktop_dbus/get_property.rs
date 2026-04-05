use crate::{
    dbus::types::{OutgoingMessage, Value},
    utils::StringRef,
};

#[derive(Debug)]
pub(crate) struct GetProperty;

impl GetProperty {
    pub(crate) fn build(
        destination: impl Into<StringRef>,
        path: impl Into<StringRef>,
        interface: impl Into<StringRef>,
        property: impl Into<StringRef>,
    ) -> OutgoingMessage {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: path.into(),
            member: StringRef::new("Get"),
            interface: Some(StringRef::new("org.freedesktop.DBus.Properties")),
            destination: Some(destination.into()),
            sender: None,
            unix_fds: None,
            body: vec![
                Value::StringRef(interface.into()),
                Value::StringRef(property.into()),
            ],
        }
    }
}
