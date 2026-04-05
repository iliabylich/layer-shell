use crate::{
    dbus::types::{OutgoingMessage, Value},
    utils::StringRef,
};

#[derive(Debug)]
pub(crate) struct GetProperty {
    destination: StringRef,
    path: StringRef,
    interface: StringRef,
    property: StringRef,
}

impl GetProperty {
    pub(crate) const fn new(
        destination: StringRef,
        path: StringRef,
        interface: StringRef,
        property: StringRef,
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
            member: StringRef::new("Get"),
            interface: Some(StringRef::new("org.freedesktop.DBus.Properties")),
            destination: Some(value.destination),
            sender: None,
            unix_fds: None,
            body: vec![
                Value::StringRef(value.interface),
                Value::StringRef(value.property),
            ],
        }
    }
}
