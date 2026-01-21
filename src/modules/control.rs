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
use anyhow::Result;

pub(crate) struct Control;

impl Control {
    pub(crate) fn new() -> Box<Self> {
        Box::new(Self)
    }

    pub(crate) fn init(&mut self, dbus: &mut DBus) {
        let mut message: Message = RequestName::new("org.me.LayerShellControl").into();
        dbus.enqueue(&mut message);
    }

    fn reply_ok(dbus: &mut DBus, serial: u32, destination: &str) {
        let mut reply = Message::new_method_return_no_body(serial, destination);
        dbus.enqueue(&mut reply);
    }

    fn reply_err(dbus: &mut DBus, serial: u32, destination: &str) {
        let mut reply = Message::new_err_no_method(serial, destination);
        dbus.enqueue(&mut reply);
    }

    fn try_parse_introspect_req(message: &Message) -> Result<(String, u32)> {
        let IntrospectRequest {
            destination,
            path,
            sender,
            serial,
        } = IntrospectRequest::try_from(message)?;

        destination_is!(destination, "org.me.LayerShellControl");
        path_is!(path, "/");
        Ok((sender.to_string(), serial))
    }

    fn try_parse_control_req<'a>(message: &'a Message<'a>) -> Result<(&'a str, &'a str, u32)> {
        message_is!(
            message,
            Message::MethodCall {
                path,
                member,
                interface,
                destination,
                body,
                sender: Some(sender),
                serial,
                ..
            }
        );

        path_is!(path, "/");
        destination_is!(destination.as_deref(), Some("org.me.LayerShellControl"));
        interface_is!(interface.as_deref(), Some("org.me.LayerShellControl"));
        body_is!(body, []);

        Ok((member.as_ref(), sender.as_ref(), *serial))
    }

    pub(crate) fn on_message(
        &mut self,
        message: &Message,
        dbus: &mut DBus,
    ) -> Option<ControlRequest> {
        if let Ok((sender, serial)) = Self::try_parse_introspect_req(message) {
            let mut reply: Message = IntrospectResponse::new(serial, &sender, INTROSPECTION).into();
            dbus.enqueue(&mut reply);
            return None;
        }

        if let Ok((member, sender, serial)) = Self::try_parse_control_req(message) {
            if let Ok(control_req) = ControlRequest::try_from(member) {
                Self::reply_ok(dbus, serial, sender);
                return Some(control_req);
            } else {
                Self::reply_err(dbus, serial, sender);
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

#[derive(Debug)]
pub(crate) enum ControlRequest {
    CapsLockToggled,
    Exit,
    ReloadStyles,
    ToggleSessionScreen,
}

impl TryFrom<&str> for ControlRequest {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "CapsLockToggled" => Ok(ControlRequest::CapsLockToggled),
            "Exit" => Ok(ControlRequest::Exit),
            "ReloadStyles" => Ok(ControlRequest::ReloadStyles),
            "ToggleSessionScreen" => Ok(ControlRequest::ToggleSessionScreen),
            _ => Err(()),
        }
    }
}
