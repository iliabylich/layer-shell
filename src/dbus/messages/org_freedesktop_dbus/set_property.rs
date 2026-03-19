use crate::{
    dbus::{OutgoingMessage, types::Value},
    ffi::ShortString,
};
use std::borrow::Cow;

pub(crate) struct SetProperty<'a> {
    destination: ShortString,
    path: ShortString,
    interface: Cow<'a, str>,
    property: Cow<'a, str>,
    value: Value<'a>,
}

impl<'a> SetProperty<'a> {
    pub(crate) fn new(
        destination: ShortString,
        path: ShortString,
        interface: impl Into<Cow<'a, str>>,
        property: impl Into<Cow<'a, str>>,
        value: Value<'a>,
    ) -> Self {
        Self {
            destination,
            path,
            interface: interface.into(),
            property: property.into(),
            value,
        }
    }
}

impl<'a> From<SetProperty<'a>> for OutgoingMessage<'a> {
    fn from(message: SetProperty<'a>) -> Self {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: message.path,
            member: ShortString::from("Set"),
            interface: Some(ShortString::from("org.freedesktop.DBus.Properties")),
            destination: Some(message.destination),
            sender: None,
            unix_fds: None,
            body: vec![
                Value::String(message.interface),
                Value::String(message.property),
                Value::Variant(Box::new(message.value)),
            ],
        }
    }
}
