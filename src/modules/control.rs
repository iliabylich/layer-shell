use crate::dbus::{
    DBus, Message,
    messages::{
        body_is, destination_is, interface_is,
        introspect::{IntrospectRequest, IntrospectResponse},
        message_is,
        org_freedesktop_dbus::RequestName,
        path_is,
    },
};
use anyhow::{Result, bail};

pub(crate) struct Control;

impl Control {
    pub(crate) fn new() -> Box<Self> {
        Box::new(Self)
    }

    pub(crate) fn init(&mut self, dbus: &mut DBus) -> Result<()> {
        let mut message: Message = RequestName::new("org.me.LayerShellControl").into();
        dbus.enqueue(&mut message)
    }

    pub(crate) fn on_message(
        &mut self,
        message: &Message,
        dbus: &mut DBus,
    ) -> Result<Option<ControlRequest>> {
        if let Ok((sender, serial)) = try_parse_introspect_req(message) {
            let mut reply: Message =
                IntrospectResponse::new(serial, sender, INTROSPECTION.to_string()).into();
            dbus.enqueue(&mut reply)?;
            return Ok(None);
        }

        if let Ok((member, sender, serial)) = try_parse_control_req(message) {
            if let Ok(control_req) = ControlRequest::try_parse(member) {
                let mut reply = Message::new_method_return_no_body(serial, sender);
                dbus.enqueue(&mut reply)?;
                return Ok(Some(control_req));
            } else {
                let mut reply = Message::new_err_no_method(serial, sender);
                dbus.enqueue(&mut reply)?;
                return Ok(None);
            }
        }

        Ok(None)
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

fn try_parse_introspect_req<'a>(message: &'a Message<'a>) -> Result<(&'a str, u32)> {
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

fn try_parse_control_req<'a>(message: &'a Message<'a>) -> Result<(&'a str, &'a str, u32)> {
    message_is!(
        message,
        Message::MethodCall {
            path,
            member,
            interface: Some(interface),
            destination: Some(destination),
            body,
            sender: Some(sender),
            serial,
            ..
        }
    );

    path_is!(path, "/");
    destination_is!(destination.as_ref(), "org.me.LayerShellControl");
    interface_is!(interface.as_ref(), "org.me.LayerShellControl");
    body_is!(body, []);

    Ok((member.as_ref(), sender.as_ref(), *serial))
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
