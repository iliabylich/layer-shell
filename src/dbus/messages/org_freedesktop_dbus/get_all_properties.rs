use crate::dbus::types::{Message, Value};
use std::borrow::Cow;

#[derive(Debug)]
pub(crate) struct GetAllProperties {
    destination: Cow<'static, str>,
    path: Cow<'static, str>,
    interface: Cow<'static, str>,
}
impl GetAllProperties {
    pub(crate) fn new(
        destination: Cow<'static, str>,
        path: Cow<'static, str>,
        interface: Cow<'static, str>,
    ) -> Self {
        Self {
            destination,
            path,
            interface,
        }
    }
}
impl From<GetAllProperties> for Message {
    fn from(value: GetAllProperties) -> Message {
        Message::MethodCall {
            serial: 0,
            path: value.path.clone(),
            member: Cow::Borrowed("GetAll"),
            interface: Some(Cow::Borrowed("org.freedesktop.DBus.Properties")),
            destination: Some(value.destination.clone()),
            sender: None,
            unix_fds: None,
            body: vec![Value::String(value.interface.to_string())],
        }
    }
}
