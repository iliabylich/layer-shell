use crate::{
    dbus::{OutgoingMessage, types::Value},
    ffi::ShortString,
};

pub(crate) struct SetProperty {
    destination: ShortString,
    path: ShortString,
    interface: ShortString,
    property: ShortString,
    value: Value,
}

impl SetProperty {
    pub(crate) fn new(
        destination: ShortString,
        path: ShortString,
        interface: ShortString,
        property: ShortString,
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
            member: ShortString::from("Set"),
            interface: Some(ShortString::from("org.freedesktop.DBus.Properties")),
            destination: Some(message.destination),
            sender: None,
            unix_fds: None,
            body: vec![
                Value::ShortString(message.interface),
                Value::ShortString(message.property),
                Value::Variant(Box::new(message.value)),
            ],
        }
    }
}
