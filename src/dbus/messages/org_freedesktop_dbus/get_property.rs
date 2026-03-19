use crate::{
    dbus::types::{OutgoingMessage, Value},
    ffi::ShortString,
};
use std::borrow::Cow;

#[derive(Debug)]
pub(crate) struct GetProperty<'a> {
    destination: ShortString,
    path: Cow<'a, str>,
    interface: Cow<'a, str>,
    property: Cow<'a, str>,
}

impl<'a> GetProperty<'a> {
    pub(crate) fn new(
        destination: ShortString,
        path: impl Into<Cow<'a, str>>,
        interface: impl Into<Cow<'a, str>>,
        property: impl Into<Cow<'a, str>>,
    ) -> Self {
        Self {
            destination,
            path: path.into(),
            interface: interface.into(),
            property: property.into(),
        }
    }
}

impl<'a> From<GetProperty<'a>> for OutgoingMessage<'a> {
    fn from(value: GetProperty<'a>) -> Self {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: value.path,
            member: Cow::Borrowed("Get"),
            interface: Some(Cow::Borrowed("org.freedesktop.DBus.Properties")),
            destination: Some(value.destination),
            sender: None,
            unix_fds: None,
            body: vec![
                Value::String(value.interface),
                Value::String(value.property),
            ],
        }
    }
}
