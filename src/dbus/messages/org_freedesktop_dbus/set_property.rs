use crate::dbus::{Message, types::Value};
use std::borrow::Cow;

pub(crate) struct SetProperty<'a> {
    destination: Cow<'a, str>,
    path: Cow<'a, str>,
    interface: Cow<'a, str>,
    property: Cow<'a, str>,
    value: Value<'a>,
}

impl<'a> SetProperty<'a> {
    pub(crate) fn new(
        destination: impl Into<Cow<'a, str>>,
        path: impl Into<Cow<'a, str>>,
        interface: impl Into<Cow<'a, str>>,
        property: impl Into<Cow<'a, str>>,
        value: Value<'a>,
    ) -> Self {
        Self {
            destination: destination.into(),
            path: path.into(),
            interface: interface.into(),
            property: property.into(),
            value,
        }
    }
}

impl<'a> From<SetProperty<'a>> for Message<'a> {
    fn from(message: SetProperty<'a>) -> Self {
        Message::MethodCall {
            serial: 0,
            path: message.path.clone(),
            member: Cow::Borrowed("Set"),
            interface: Some(Cow::Borrowed("org.freedesktop.DBus.Properties")),
            destination: Some(message.destination.clone()),
            sender: None,
            unix_fds: None,
            body: vec![
                Value::String(message.interface.clone()),
                Value::String(message.property.clone()),
                Value::Variant(Box::new(message.value)),
            ],
        }
    }
}
