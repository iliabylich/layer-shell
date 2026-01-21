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

pub(crate) struct StatusNotifierWatcher {
    reply_serial: Option<u32>,
}

impl StatusNotifierWatcher {
    pub(crate) fn new() -> Self {
        Self { reply_serial: None }
    }

    pub(crate) fn request(&mut self, dbus: &mut DBus) {
        let mut message: Message = RequestName::new("org.kde.StatusNotifierWatcher").into();
        dbus.enqueue(&mut message);
        self.reply_serial = Some(message.serial());
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

        destination_is!(destination, "org.kde.StatusNotifierWatcher");
        path_is!(path, "/");
        Ok((sender.to_string(), serial))
    }

    fn try_parse_sni_req<'a>(message: &'a Message<'a>) -> Result<(&'a str, &'a str, u32)> {
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

        path_is!(path, "/StatusNotifierWatcher");
        destination_is!(
            destination.as_deref(),
            Some("org.kde.StatusNotifierWatcher")
        );
        interface_is!(interface.as_deref(), Some("org.kde.StatusNotifierWatcher"));
        body_is!(body, []);

        Ok((member.as_ref(), sender.as_ref(), *serial))
    }

    pub(crate) fn init(&mut self, dbus: &mut DBus) {
        self.request(dbus);
    }

    pub(crate) fn on_message(&mut self, dbus: &mut DBus, message: &Message) {
        if let Ok((sender, serial)) = Self::try_parse_introspect_req(message) {
            let mut reply: Message = IntrospectResponse::new(serial, &sender, INTROSPECTION).into();
            dbus.enqueue(&mut reply);
            return;
        }

        if let Ok((_member, _sender, _serial)) = Self::try_parse_sni_req(message) {}

        println!("{message:?}")
    }
}

const INTROSPECTION: &str = r#"
<!DOCTYPE node PUBLIC "-//freedesktop//DTD D-BUS Object Introspection 1.0//EN" "http://www.freedesktop.org/standards/dbus/1.0/introspect.dtd">
<node>
    <interface name="org.kde.StatusNotifierWatcher">

        <method name="RegisterStatusNotifierItem">
            <arg name="service" type="s" direction="in" />
        </method>

        <method name="RegisterStatusNotifierHost">
            <arg name="service" type="s" direction="in" />
        </method>

        <property name="RegisteredStatusNotifierItems" type="as" access="read" />
        <property name="IsStatusNotifierHostRegistered" type="b" access="read" />
        <property name="ProtocolVersion" type="i" access="read" />

        <signal name="StatusNotifierItemRegistered">
            <arg type="s" />
        </signal>
        <signal name="StatusNotifierItemUnregistered">
            <arg type="s" />
        </signal>
        <signal name="StatusNotifierHostRegistered" />
        <signal name="StatusNotifierHostUnregistered" />

    </interface>
</node>
"#;

enum KSNIRequest<'a> {
    RegisteredStatusNotifierItem { service: &'a str },
    RegisterStatusNotifierHost { service: &'a str },
}
