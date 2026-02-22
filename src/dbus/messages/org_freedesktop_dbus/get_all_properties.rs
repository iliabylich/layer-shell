use crate::dbus::types::{Message, Value};
use std::borrow::Cow;

#[derive(Debug)]
pub(crate) struct GetAllProperties<'a> {
    destination: Cow<'a, str>,
    path: Cow<'a, str>,
    interface: Cow<'a, str>,
}
impl<'a> GetAllProperties<'a> {
    pub(crate) fn new(
        destination: impl Into<Cow<'a, str>>,
        path: impl Into<Cow<'a, str>>,
        interface: impl Into<Cow<'a, str>>,
    ) -> Self {
        Self {
            destination: destination.into(),
            path: path.into(),
            interface: interface.into(),
        }
    }
}
impl<'a> From<GetAllProperties<'a>> for Message<'a> {
    fn from(value: GetAllProperties<'a>) -> Self {
        Message::MethodCall {
            serial: 0,
            path: value.path,
            member: Cow::Borrowed("GetAll"),
            interface: Some(Cow::Borrowed("org.freedesktop.DBus.Properties")),
            destination: Some(value.destination),
            sender: None,
            unix_fds: None,
            body: vec![Value::String(value.interface)],
        }
    }
}
