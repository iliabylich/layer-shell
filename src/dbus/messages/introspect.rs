use super::{interface_is, member_is, path_is};
use crate::dbus::{
    decoder::{IncomingMessage, MessageType},
    types::{Message, Value},
};
use anyhow::{Context as _, Result, ensure};
use std::borrow::Cow;

#[derive(Debug)]
pub(crate) struct IntrospectRequest<'a> {
    pub(crate) serial: u32,
    pub(crate) destination: &'a str,
    pub(crate) path: &'a str,
    pub(crate) sender: &'a str,
}

impl<'a> TryFrom<IncomingMessage<'a>> for IntrospectRequest<'a> {
    type Error = anyhow::Error;

    fn try_from(message: IncomingMessage<'a>) -> Result<Self> {
        ensure!(message.message_type == MessageType::MethodCall);

        let serial = message.serial;
        let path = message.path.context("no Path")?;
        let member = message.member.context("no Member")?;
        let interface = message.interface.context("no Interface")?;
        let destination = message.destination.context("no Destination")?;
        let sender = message.sender.context("no Sender")?;
        ensure!(message.body.is_none());

        path_is!(path, "/");
        member_is!(member, "Introspect");
        interface_is!(interface, "org.freedesktop.DBus.Introspectable");

        Ok(Self {
            serial,
            destination,
            path,
            sender,
        })
    }
}

pub(crate) struct IntrospectResponse<'a> {
    reply_serial: u32,
    destination: &'a str,
    xml: String,
}

impl<'a> IntrospectResponse<'a> {
    pub(crate) fn new(reply_serial: u32, destination: &'a str, xml: String) -> Self {
        Self {
            reply_serial,
            destination,
            xml,
        }
    }
}

impl<'a> From<IntrospectResponse<'a>> for Message<'a> {
    fn from(value: IntrospectResponse<'a>) -> Self {
        Message::MethodReturn {
            serial: 0,
            reply_serial: value.reply_serial,
            destination: Some(Cow::Borrowed(value.destination)),
            sender: None,
            unix_fds: None,
            body: vec![Value::String(Cow::Owned(value.xml))],
        }
    }
}
