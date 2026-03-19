use crate::{
    dbus::types::{OutgoingMessage, Value},
    ffi::ShortString,
};

#[derive(Debug)]
pub(crate) struct GetAllProperties {
    destination: ShortString,
    path: ShortString,
    interface: ShortString,
}
impl GetAllProperties {
    pub(crate) fn new(destination: ShortString, path: ShortString, interface: ShortString) -> Self {
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
            member: ShortString::from("GetAll"),
            interface: Some(ShortString::from("org.freedesktop.DBus.Properties")),
            destination: Some(value.destination),
            sender: None,
            unix_fds: None,
            body: vec![Value::ShortString(value.interface)],
        }
    }
}
