use crate::{
    dbus::types::{OutgoingMessage, Value},
    ffi::ShortString,
};

#[derive(Debug)]
pub(crate) struct GetProperty {
    destination: ShortString,
    path: ShortString,
    interface: ShortString,
    property: ShortString,
}

impl GetProperty {
    pub(crate) fn new(
        destination: ShortString,
        path: ShortString,
        interface: ShortString,
        property: ShortString,
    ) -> Self {
        Self {
            destination,
            path,
            interface,
            property,
        }
    }
}

impl From<GetProperty> for OutgoingMessage {
    fn from(value: GetProperty) -> Self {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: value.path,
            member: ShortString::from("Get"),
            interface: Some(ShortString::from("org.freedesktop.DBus.Properties")),
            destination: Some(value.destination),
            sender: None,
            unix_fds: None,
            body: vec![
                Value::ShortString(value.interface),
                Value::ShortString(value.property),
            ],
        }
    }
}
