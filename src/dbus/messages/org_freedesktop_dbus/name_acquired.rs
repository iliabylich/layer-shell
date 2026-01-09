use super::{body_is, interface_is, message_is, path_is};
use crate::dbus::types::{Message, Value};
use anyhow::Result;
use std::borrow::Cow;

#[derive(Debug)]
pub(crate) struct NameAcquired<'a> {
    pub(crate) name: Cow<'a, str>,
}
impl<'a> TryFrom<&'a Message> for NameAcquired<'a> {
    type Error = anyhow::Error;

    fn try_from(message: &'a Message) -> Result<Self> {
        message_is!(
            message,
            Message::Signal {
                path,
                interface,
                body,
                ..
            }
        );

        interface_is!(interface, "org.freedesktop.DBus");
        path_is!(path, "/org/freedesktop/DBus");
        body_is!(body, [Value::String(name)]);

        Ok(Self {
            name: Cow::Borrowed(name.as_str()),
        })
    }
}
