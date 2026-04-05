use crate::{
    dbus::{OutgoingMessage, types::Value},
    utils::StringRef,
};

pub(crate) struct SetProperty {
    destination: StringRef,
    path: StringRef,
    interface: StringRef,
    property: StringRef,
    value: Value,
}

impl SetProperty {
    pub(crate) const fn new(
        destination: StringRef,
        path: StringRef,
        interface: StringRef,
        property: StringRef,
        value: Value,
    ) -> Self {
        Self {
            destination,
            path,
            interface,
            property,
            value,
        }
    }
}

impl From<SetProperty> for OutgoingMessage {
    fn from(message: SetProperty) -> Self {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: message.path,
            member: StringRef::new("Set"),
            interface: Some(StringRef::new("org.freedesktop.DBus.Properties")),
            destination: Some(message.destination),
            sender: None,
            unix_fds: None,
            body: vec![
                Value::StringRef(message.interface),
                Value::StringRef(message.property),
                Value::Variant(Box::new(message.value)),
            ],
        }
    }
}
