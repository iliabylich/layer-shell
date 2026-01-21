use crate::dbus::types::{Message, Value};
use std::borrow::Cow;

#[derive(Debug)]
pub(crate) struct GetAllProperties<'a> {
    destination: &'a str,
    path: &'a str,
    interface: &'a str,
}
impl<'a> GetAllProperties<'a> {
    pub(crate) fn new(destination: &'a str, path: &'a str, interface: &'a str) -> Self {
        Self {
            destination,
            path,
            interface,
        }
    }
}
impl<'a> From<GetAllProperties<'a>> for Message<'a> {
    fn from(value: GetAllProperties<'a>) -> Self {
        Message::MethodCall {
            serial: 0,
            path: Cow::Borrowed(value.path),
            member: Cow::Borrowed("GetAll"),
            interface: Some(Cow::Borrowed("org.freedesktop.DBus.Properties")),
            destination: Some(Cow::Borrowed(value.destination)),
            sender: None,
            unix_fds: None,
            body: vec![Value::String(Cow::Borrowed(value.interface))],
        }
    }
}
