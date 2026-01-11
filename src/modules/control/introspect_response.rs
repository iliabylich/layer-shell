use crate::dbus::{Message, messages::introspect::IntrospectResponse};

pub(crate) struct ControlIntrospectResponse<'a> {
    reply_serial: u32,
    destination: &'a str,
}

impl<'a> ControlIntrospectResponse<'a> {
    pub(crate) fn new(reply_serial: u32, destination: &'a str) -> Self {
        Self {
            reply_serial,
            destination,
        }
    }
}

impl<'a> From<ControlIntrospectResponse<'a>> for Message {
    fn from(
        ControlIntrospectResponse {
            reply_serial,
            destination,
        }: ControlIntrospectResponse,
    ) -> Self {
        IntrospectResponse::new(reply_serial, destination, INTROSPECTION).into()
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
