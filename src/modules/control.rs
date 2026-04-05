use crate::{
    dbus::{
        OutgoingMessage,
        decoder::{IncomingMessage, MessageType},
        messages::{
            destination_is, interface_is,
            introspect::{IntrospectRequest, IntrospectResponse},
            org_freedesktop_dbus::RequestName,
            path_is,
        },
    },
    sansio::SessionDBusQueue,
    utils::StringRef,
};
use anyhow::{Context, Result, bail, ensure};

pub(crate) struct Control;

impl Control {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn init(&mut self) {
        let message = RequestName::build("org.me.LayerShellControl");
        SessionDBusQueue::push_back(message);
    }

    pub(crate) fn on_message(&mut self, message: IncomingMessage<'_>) -> Option<ControlRequest> {
        if let Ok((sender, serial)) = try_parse_introspect_req(message) {
            let reply = IntrospectResponse::new(serial, sender, INTROSPECTION.to_string());
            SessionDBusQueue::push_back(reply);
            return None;
        }

        if let Ok((member, sender, serial)) = try_parse_control_req(message) {
            if let Ok(control_req) = ControlRequest::try_parse(member) {
                let reply = OutgoingMessage::new_method_return_no_body(serial, sender);
                SessionDBusQueue::push_back(reply);
                return Some(control_req);
            } else {
                let reply = OutgoingMessage::new_err_no_method(serial, sender);
                SessionDBusQueue::push_back(reply);
                return None;
            }
        }

        None
    }
}

const INTROSPECTION: &str = r#"
<?xml version="1.0" encoding="UTF-8"?>
<node>
    <interface name="org.me.LayerShellControl">
        <method name="CapsLockToggled"></method>
        <method name="Exit"></method>
        <method name="ReloadStyles"></method>
        <method name="ToggleSessionScreen"></method>
    </interface>
</node>
"#;

fn try_parse_introspect_req(message: IncomingMessage) -> Result<(StringRef, u32)> {
    let IntrospectRequest {
        destination,
        path,
        sender,
        serial,
    } = IntrospectRequest::try_from(message)?;

    destination_is!(destination, "org.me.LayerShellControl");
    path_is!(path, "/");
    Ok((sender, serial))
}

fn try_parse_control_req<'a>(message: IncomingMessage<'a>) -> Result<(&'a str, &'a str, u32)> {
    ensure!(message.message_type == MessageType::MethodCall);
    let serial = message.serial;
    let path = message.path.context("no Path")?;
    let member = message.member.context("no Member")?;
    let interface = message.interface.context("no Interface")?;
    let destination = message.destination.context("no Destination")?;
    let sender = message.sender.context("no Sender")?;

    path_is!(path, "/");
    destination_is!(destination, "org.me.LayerShellControl");
    interface_is!(interface, "org.me.LayerShellControl");
    ensure!(message.body.is_none());

    Ok((member, sender, serial))
}

#[derive(Debug)]
pub(crate) enum ControlRequest {
    CapsLockToggled,
    Exit,
    ReloadStyles,
    ToggleSessionScreen,
}

impl ControlRequest {
    fn try_parse(s: &str) -> Result<Self> {
        match s {
            "CapsLockToggled" => Ok(ControlRequest::CapsLockToggled),
            "Exit" => Ok(ControlRequest::Exit),
            "ReloadStyles" => Ok(ControlRequest::ReloadStyles),
            "ToggleSessionScreen" => Ok(ControlRequest::ToggleSessionScreen),
            _ => bail!("unsupported method"),
        }
    }
}
