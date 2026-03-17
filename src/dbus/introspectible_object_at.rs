use crate::dbus::{
    decoder::{IncomingMessage, MessageType, Value},
    messages::{destination_is, value_is},
};
use anyhow::{Context as _, Result, bail, ensure};

pub(crate) struct IntrospectibleObjectAt {
    destination: &'static str,
}

impl IntrospectibleObjectAt {
    pub(crate) fn new(destination: &'static str) -> Self {
        Self { destination }
    }

    pub(crate) fn handle<'a>(
        &self,
        message: IncomingMessage<'a>,
    ) -> Result<(u32, &'a str, IntrospectibleObjectAtRequest<'a>)> {
        ensure!(message.message_type == MessageType::MethodCall);

        let serial = message.serial;
        let path = message.path.context("no Path")?;
        let member = message.member.context("no Member")?;
        let interface = message.interface.context("no Interface")?;
        let destination = message.destination.context("no Destination")?;
        let sender = message.sender.context("no Sender")?;
        let mut body = message.body.context("no Body")?;

        destination_is!(destination, self.destination);

        let req = match interface {
            "org.freedesktop.DBus.Introspectable" => match member {
                "Introspect" => IntrospectibleObjectAtRequest::Introspect { path },
                _ => bail!("unknown member {member:?}"),
            },

            "org.freedesktop.DBus.Peer" => match member {
                "GetMachinId" => IntrospectibleObjectAtRequest::GetMachineId,
                "Ping" => IntrospectibleObjectAtRequest::Ping,
                _ => bail!("unknown member {member:?}"),
            },

            "org.freedesktop.DBus.Properties" => match member {
                "Get" => {
                    let interface = body.try_next()?.context("no Interface")?;
                    value_is!(interface, Value::String(interface));

                    let property_name = body.try_next()?.context("no PropertyName")?;
                    value_is!(property_name, Value::String(property_name));

                    IntrospectibleObjectAtRequest::GetProperty {
                        path,
                        interface,
                        property_name,
                    }
                }
                "GetAll" => {
                    let interface = body.try_next()?.context("no Interface")?;
                    value_is!(interface, Value::String(interface));

                    IntrospectibleObjectAtRequest::GetAllProperties { path, interface }
                }
                "Set" => IntrospectibleObjectAtRequest::SetProperty,
                _ => bail!("unknown member {member:?}"),
            },

            _ => bail!("unknown interface {interface:?}"),
        };

        Ok((serial, sender, req))
    }
}

#[derive(Debug)]
pub(crate) enum IntrospectibleObjectAtRequest<'a> {
    Introspect {
        path: &'a str,
    },

    Ping,
    GetMachineId,

    GetProperty {
        path: &'a str,
        interface: &'a str,
        property_name: &'a str,
    },
    GetAllProperties {
        path: &'a str,
        interface: &'a str,
    },
    SetProperty,
}
