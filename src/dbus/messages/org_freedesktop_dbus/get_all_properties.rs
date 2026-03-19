use crate::{
    dbus::types::{OutgoingMessage, Value},
    ffi::ShortString,
};
use std::borrow::Cow;

#[derive(Debug)]
pub(crate) struct GetAllProperties<'a> {
    destination: ShortString,
    path: Cow<'a, str>,
    interface: Cow<'a, str>,
}
impl<'a> GetAllProperties<'a> {
    pub(crate) fn new(
        destination: ShortString,
        path: impl Into<Cow<'a, str>>,
        interface: impl Into<Cow<'a, str>>,
    ) -> Self {
        Self {
            destination,
            path: path.into(),
            interface: interface.into(),
        }
    }
}
impl<'a> From<GetAllProperties<'a>> for OutgoingMessage<'a> {
    fn from(value: GetAllProperties<'a>) -> Self {
        OutgoingMessage::MethodCall {
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
