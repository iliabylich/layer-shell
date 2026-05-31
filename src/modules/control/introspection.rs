use crate::{
    modules::SessionDBus,
    utils::{StringRef, StringRefExt as _},
};
use anyhow::Result;
use dbus::{
    IncomingMessage,
    messages::introspect::{IntrospectRequest, IntrospectResponse},
};

pub(crate) struct Introspection;

impl Introspection {
    pub(crate) fn handle(message: IncomingMessage<'_>) -> Result<bool> {
        let Some((sender, serial)) = parse_request(message) else {
            return Ok(false);
        };

        reply(sender.as_str(), serial)?;
        Ok(true)
    }
}

fn parse_request(message: IncomingMessage) -> Option<(StringRef, u32)> {
    let IntrospectRequest {
        destination,
        path,
        sender,
        serial,
    } = IntrospectRequest::try_parse(message)?;

    if destination != "org.me.LayerShellControl" {
        return None;
    }
    if path != "/" {
        return None;
    }
    Some((StringRef::new(sender), serial))
}

fn reply(destination: &str, reply_serial: u32) -> Result<()> {
    let mut buf = [0; 2_048];
    let encoded =
        IntrospectResponse::encode(&mut buf, reply_serial, destination, INTROSPECTION_XML)?;
    SessionDBus::queue().push_raw_buf(encoded);
    Ok(())
}

const INTROSPECTION_XML: &str = r#"
<?xml version="1.0" encoding="UTF-8"?>
<node>
    <interface name="org.me.LayerShellControl">
        <method name="Exit"></method>
        <method name="ToggleSessionScreen"></method>
    </interface>
</node>
"#;
