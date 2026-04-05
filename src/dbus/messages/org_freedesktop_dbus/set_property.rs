use crate::{
    dbus::{OutgoingMessage, types::Value},
    utils::StringRef,
};

pub(crate) struct SetProperty;

impl SetProperty {
    pub(crate) fn build(
        destination: impl Into<StringRef>,
        path: impl Into<StringRef>,
        interface: impl Into<StringRef>,
        property: impl Into<StringRef>,
        value: Value,
    ) -> OutgoingMessage {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: path.into(),
            member: StringRef::new("Set"),
            interface: Some(StringRef::new("org.freedesktop.DBus.Properties")),
            destination: Some(destination.into()),
            sender: None,
            unix_fds: None,
            body: vec![
                Value::StringRef(interface.into()),
                Value::StringRef(property.into()),
                Value::Variant(Box::new(value)),
            ],
        }
    }
}
