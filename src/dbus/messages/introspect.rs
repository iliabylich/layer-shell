use super::{body_is, interface_is, member_is, message_is, path_is};
use crate::dbus::types::{Message, Value};
use anyhow::Result;
use std::borrow::Cow;

#[derive(Debug)]
pub(crate) struct IntrospectRequest<'a> {
    pub(crate) serial: u32,
    pub(crate) destination: Cow<'a, str>,
    pub(crate) path: Cow<'a, str>,
    pub(crate) sender: Cow<'a, str>,
}

impl<'a> TryFrom<&'a Message> for IntrospectRequest<'a> {
    type Error = anyhow::Error;

    fn try_from(message: &'a Message) -> Result<Self> {
        message_is!(
            message,
            Message::MethodCall {
                serial,
                path,
                member,
                interface: Some(interface),
                destination: Some(destination),
                sender: Some(sender),
                body,
                ..
            }
        );

        path_is!(path, "/");
        member_is!(member, "Introspect");
        interface_is!(interface, "org.freedesktop.DBus.Introspectable");
        body_is!(body, []);

        Ok(Self {
            serial: *serial,
            destination: destination.clone(),
            path: path.clone(),
            sender: sender.clone(),
        })
    }
}

pub(crate) struct IntrospectResponse<'a> {
    reply_serial: u32,
    destination: &'a str,
    xml: &'static str,
}

impl<'a> IntrospectResponse<'a> {
    pub(crate) fn new(reply_serial: u32, destination: &'a str, xml: &'static str) -> Self {
        Self {
            reply_serial,
            destination,
            xml,
        }
    }
}

impl<'a> From<IntrospectResponse<'a>> for Message {
    fn from(value: IntrospectResponse<'a>) -> Message {
        Message::MethodReturn {
            serial: 0,
            reply_serial: value.reply_serial,
            destination: Some(Cow::Owned(value.destination.to_string())),
            sender: None,
            unix_fds: None,
            body: vec![Value::String(value.xml.to_string())],
        }
    }
}
