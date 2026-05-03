use crate::{modules::SessionDBus, utils::StringRef};
use anyhow::{Context, Result, bail, ensure};
use mini_sansio_dbus::{
    IncomingMessage, MessageType, OutgoingMessage, destination_is, interface_is,
    messages::{
        introspect::{IntrospectRequest, IntrospectResponse},
        org_freedesktop_dbus::RequestName,
    },
    path_is,
};

pub(crate) struct Control;

impl Control {
    pub(crate) fn init() {
        let message = RequestName::build("org.me.LayerShellControl");
        SessionDBus::queue().push_back(message);
    }

    pub(crate) fn on_message(message: IncomingMessage<'_>) -> Option<ControlRequest> {
        if let Ok((sender, serial)) = try_parse_introspect_req(message) {
            let reply =
                IntrospectResponse::build(serial, sender.as_str(), INTROSPECTION.to_string());
            SessionDBus::queue().push_back(reply);
            None
        } else if let Ok((member, sender, serial)) = try_parse_control_req(message) {
            let Ok(control_req) = ControlRequest::try_parse(member) else {
                let reply = OutgoingMessage::new_err_no_method(serial, sender);
                SessionDBus::queue().push_back(reply);
                return None;
            };

            let reply = OutgoingMessage::new_method_return_no_body(serial, sender);
            SessionDBus::queue().push_back(reply);
            Some(control_req)
        } else {
            None
        }
    }
}

const INTROSPECTION: &str = r#"
<?xml version="1.0" encoding="UTF-8"?>
<node>
    <interface name="org.me.LayerShellControl">
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
    Ok((StringRef::new(&sender)?, serial))
}

fn try_parse_control_req(message: IncomingMessage<'_>) -> Result<(&str, &str, u32)> {
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

#[derive(Debug, Clone, Copy)]
pub(crate) enum ControlRequest {
    Exit,
    ReloadStyles,
    ToggleSessionScreen,
}

impl ControlRequest {
    fn try_parse(s: &str) -> Result<Self> {
        match s {
            "Exit" => Ok(Self::Exit),
            "ReloadStyles" => Ok(Self::ReloadStyles),
            "ToggleSessionScreen" => Ok(Self::ToggleSessionScreen),
            _ => bail!("unsupported method"),
        }
    }
}
