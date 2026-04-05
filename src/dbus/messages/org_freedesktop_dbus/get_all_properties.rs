use crate::{
    dbus::types::{OutgoingMessage, Value},
    utils::StringRef,
};

#[derive(Debug)]
pub(crate) struct GetAllProperties {
    destination: StringRef,
    path: StringRef,
    interface: StringRef,
}

impl GetAllProperties {
    pub(crate) const fn new(destination: StringRef, path: StringRef, interface: StringRef) -> Self {
        Self {
            destination,
            path,
            interface,
        }
    }
}

impl From<GetAllProperties> for OutgoingMessage {
    fn from(value: GetAllProperties) -> Self {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: value.path,
            member: StringRef::new("GetAll"),
            interface: Some(StringRef::new("org.freedesktop.DBus.Properties")),
            destination: Some(value.destination),
            sender: None,
            unix_fds: None,
            body: vec![Value::StringRef(value.interface)],
        }
    }
}
