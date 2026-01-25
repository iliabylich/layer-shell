use crate::dbus::types::{Message, Value};
use std::borrow::Cow;

#[derive(Debug)]
pub(crate) struct GetProperty<'a> {
    destination: Cow<'a, str>,
    path: Cow<'a, str>,
    interface: Cow<'a, str>,
    property: Cow<'a, str>,
}

impl<'a> GetProperty<'a> {
    pub(crate) fn new(
        destination: impl Into<Cow<'a, str>>,
        path: impl Into<Cow<'a, str>>,
        interface: impl Into<Cow<'a, str>>,
        property: impl Into<Cow<'a, str>>,
    ) -> Self {
        Self {
            destination: destination.into(),
            path: path.into(),
            interface: interface.into(),
            property: property.into(),
        }
    }
}

impl<'a> From<GetProperty<'a>> for Message<'a> {
    fn from(value: GetProperty<'a>) -> Self {
        Message::MethodCall {
            serial: 0,
            path: value.path.clone(),
            member: Cow::Borrowed("Get"),
            interface: Some(Cow::Borrowed("org.freedesktop.DBus.Properties")),
            destination: Some(value.destination.clone()),
            sender: None,
            unix_fds: None,
            body: vec![
                Value::String(value.interface.clone()),
                Value::String(value.property.clone()),
            ],
        }
    }
}
