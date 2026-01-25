use crate::dbus::{
    Message,
    messages::{body_is, destination_is, message_is, value_is},
    types::Value,
};
use anyhow::{Result, bail};

pub(crate) struct IntrospectibleObjectAt {
    destination: &'static str,
}

impl IntrospectibleObjectAt {
    pub(crate) fn new(destination: &'static str) -> Self {
        Self { destination }
    }

    pub(crate) fn handle(
        &self,
        message: &Message,
    ) -> Result<(u32, String, IntrospectibleObjectAtRequest)> {
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

        destination_is!(destination, self.destination);

        let req = match interface.as_ref() {
            "org.freedesktop.DBus.Introspectable" => match member.as_ref() {
                "Introspect" => IntrospectibleObjectAtRequest::Introspect {
                    path: path.to_string(),
                },
                _ => bail!("unknown member {member:?}"),
            },

            "org.freedesktop.DBus.Peer" => match member.as_ref() {
                "GetMachinId" => IntrospectibleObjectAtRequest::GetMachineId,
                "Ping" => IntrospectibleObjectAtRequest::Ping,
                _ => bail!("unknown member {member:?}"),
            },

            "org.freedesktop.DBus.Properties" => match member.as_ref() {
                "Get" => {
                    body_is!(body, [interface, property_name]);
                    value_is!(interface, Value::String(interface));
                    value_is!(property_name, Value::String(property_name));
                    IntrospectibleObjectAtRequest::GetProperty {
                        path: path.to_string(),
                        interface: interface.to_string(),
                        property_name: property_name.to_string(),
                    }
                }
                "GetAll" => {
                    body_is!(body, [interface]);
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

        Ok((*serial, sender.to_string(), req))
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
