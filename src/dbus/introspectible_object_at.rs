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

    pub(crate) fn handle(
        &self,
        message: IncomingMessage<'_>,
    ) -> Result<(u32, String, IntrospectibleObjectAtRequest)> {
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
                "Introspect" => IntrospectibleObjectAtRequest::Introspect {
                    path: path.to_string(),
                },
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
                        path: path.to_string(),
                        interface: interface.to_string(),
                        property_name: property_name.to_string(),
                    }
                }
                "GetAll" => {
                    let interface = body.try_next()?.context("no Interface")?;
                    value_is!(interface, Value::String(interface));

                    IntrospectibleObjectAtRequest::GetAllProperties {
                        path: path.to_string(),
                        interface: interface.to_string(),
                    }
                }
                "Set" => IntrospectibleObjectAtRequest::SetProperty,
                _ => bail!("unknown member {member:?}"),
            },

            _ => bail!("unknown interface {interface:?}"),
        };

        Ok((serial, sender.to_string(), req))
    }
}

#[derive(Debug)]
pub(crate) enum IntrospectibleObjectAtRequest {
    Introspect {
        path: String,
    },

    Ping,
    GetMachineId,

    GetProperty {
        path: String,
        interface: String,
        property_name: String,
    },
    GetAllProperties {
        path: String,
        interface: String,
    },
    SetProperty,
}
