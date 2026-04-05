use super::{interface_is, member_is, path_is};
use crate::{
    dbus::{
        decoder::{IncomingMessage, MessageType},
        types::{OutgoingMessage, Value},
    },
    utils::StringRef,
};
use anyhow::{Context as _, Result, ensure};

#[derive(Debug)]
pub(crate) struct IntrospectRequest {
    pub(crate) serial: u32,
    pub(crate) destination: StringRef,
    pub(crate) path: StringRef,
    pub(crate) sender: StringRef,
}

impl TryFrom<IncomingMessage<'_>> for IntrospectRequest {
    type Error = anyhow::Error;

    fn try_from(message: IncomingMessage) -> Result<Self> {
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
            destination: StringRef::new(destination),
            path: StringRef::new(path),
            sender: StringRef::new(sender),
        })
    }
}

pub(crate) struct IntrospectResponse {
    reply_serial: u32,
    destination: StringRef,
    xml: String,
}

impl IntrospectResponse {
    pub(crate) fn new(reply_serial: u32, destination: StringRef, xml: String) -> Self {
        Self {
            reply_serial,
            destination,
            xml,
        }
    }
}

impl From<IntrospectResponse> for OutgoingMessage {
    fn from(value: IntrospectResponse) -> Self {
        OutgoingMessage::MethodReturn {
            serial: 0,
            reply_serial: value.reply_serial,
            destination: Some(value.destination),
            sender: None,
            unix_fds: None,
            body: vec![Value::LongString(value.xml)],
        }
    }
}
