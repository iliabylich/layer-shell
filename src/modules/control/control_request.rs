use crate::utils::dbus::queue::DBusQueue;
use anyhow::Result;
use dbus::{
    IncomingMessage, MessageType,
    messages::{EmptyMethodReturn, ErrorNoMethod},
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum ControlRequest {
    Exit,
    ToggleSessionScreen,
}

impl ControlRequest {
    fn try_parse(member: &str) -> Option<Self> {
        match member {
            "Exit" => Some(Self::Exit),
            "ToggleSessionScreen" => Some(Self::ToggleSessionScreen),
            _ => None,
        }
    }

    pub(crate) fn handle(message: IncomingMessage<'_>, q: &mut DBusQueue) -> Result<Option<Self>> {
        if let Some((member, sender, serial)) = try_parse_control_req(message) {
            if let Some(control_req) = Self::try_parse(member) {
                reply_ok(sender, serial, q)?;
                return Ok(Some(control_req));
            }
            reply_err(sender, serial, q)?;
            return Ok(None);
        }

        Ok(None)
    }
}

fn reply_ok(destination: &str, reply_serial: u32, q: &mut DBusQueue) -> Result<()> {
    let mut buf = [0; 200];
    let encoded = EmptyMethodReturn::encode(&mut buf, destination, reply_serial)?;
    q.push_raw(encoded);
    Ok(())
}

fn reply_err(destination: &str, reply_serial: u32, q: &mut DBusQueue) -> Result<()> {
    let mut buf = [0; 200];
    let encoded = ErrorNoMethod::encode(&mut buf, destination, reply_serial)?;
    q.push_raw(encoded);
    Ok(())
}

fn try_parse_control_req(message: IncomingMessage<'_>) -> Option<(&str, &str, u32)> {
    if message.message_type != MessageType::MethodCall {
        return None;
    }
    let serial = message.serial;
    let path = message.path?;
    let member = message.member?;
    let interface = message.interface?;
    let destination = message.destination?;
    let sender = message.sender?;

    if path != "/" {
        return None;
    }
    if destination != "org.me.LayerShellControl" {
        return None;
    }
    if interface != "org.me.LayerShellControl" {
        return None;
    }
    if message.body.is_some() {
        return None;
    }

    Some((member, sender, serial))
}
