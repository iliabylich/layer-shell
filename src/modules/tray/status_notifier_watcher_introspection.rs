use crate::modules::SessionDBus;
use mini_sansio_dbus::{
    IncomingMessage, IntrospectibleObjectAt, IntrospectibleObjectAtRequest, OutgoingCompleteType,
    OutgoingMessage, OutgoingValue,
};

pub(crate) struct StatusNotifierWatcherIntrospection {
    introspection: IntrospectibleObjectAt,
}

impl StatusNotifierWatcherIntrospection {
    pub(crate) fn new() -> Self {
        Self {
            introspection: IntrospectibleObjectAt::new("org.kde.StatusNotifierWatcher"),
        }
    }

    fn reply_ok(&self, serial: u32, destination: &str, body: Vec<OutgoingValue>) {
        let message = OutgoingMessage::MethodReturn {
            serial: 0,
            reply_serial: serial,
            destination: Some(String::from(destination)),
            sender: None,
            unix_fds: None,
            body,
        };
        SessionDBus::queue().push_back(message);
    }

    fn reply_err(&self, serial: u32, destination: &str) {
        let reply = OutgoingMessage::new_err_no_method(serial, destination);
        SessionDBus::queue().push_back(reply);
    }

    pub(crate) fn process_message(&mut self, message: IncomingMessage<'_>) -> bool {
        let Ok((serial, sender, req)) = self.introspection.handle(message) else {
            return false;
        };

        match req {
            IntrospectibleObjectAtRequest::Introspect { path } => match path {
                "/" => self.reply_ok(
                    serial,
                    sender,
                    vec![OutgoingValue::String(root_introspection_xml())],
                ),
                "/StatusNotifierWatcher" => self.reply_ok(
                    serial,
                    sender,
                    vec![OutgoingValue::String(ksni_introspection_xml())],
                ),
                _ => self.reply_err(serial, sender),
            },

            IntrospectibleObjectAtRequest::GetAllProperties { path, interface } => {
                match (path, interface) {
                    ("/StatusNotifierWatcher", "org.kde.StatusNotifierWatcher") => {
                        let body = vec![OutgoingValue::Array(
                            OutgoingCompleteType::DictEntry(
                                Box::new(OutgoingCompleteType::String),
                                Box::new(OutgoingCompleteType::Variant),
                            ),
                            vec![
                                OutgoingValue::DictEntry(
                                    Box::new(OutgoingValue::String(String::from(
                                        "ProtocolVersion",
                                    ))),
                                    Box::new(OutgoingValue::Variant(Box::new(
                                        OutgoingValue::Int32(42),
                                    ))),
                                ),
                                OutgoingValue::DictEntry(
                                    Box::new(OutgoingValue::String(String::from(
                                        "IsStatusNotifierHostRegistered",
                                    ))),
                                    Box::new(OutgoingValue::Variant(Box::new(
                                        OutgoingValue::Bool(true),
                                    ))),
                                ),
                                OutgoingValue::DictEntry(
                                    Box::new(OutgoingValue::String(String::from(
                                        "RegisteredStatusNotifierItems",
                                    ))),
                                    Box::new(OutgoingValue::Variant(Box::new(
                                        OutgoingValue::Array(OutgoingCompleteType::String, vec![]),
                                    ))),
                                ),
                            ],
                        )];
                        self.reply_ok(serial, sender, body);
                    }

                    _ => self.reply_err(serial, sender),
                }
            }

            IntrospectibleObjectAtRequest::GetProperty {
                path,
                interface,
                property_name,
            } => {
                let value = match (path, interface, property_name) {
                    (
                        "/StatusNotifierWatcher",
                        "org.kde.StatusNotifierWatcher",
                        "ProtocolVersion",
                    ) => OutgoingValue::Variant(Box::new(OutgoingValue::Int32(42))),

                    (
                        "/StatusNotifierWatcher",
                        "org.kde.StatusNotifierWatcher",
                        "IsStatusNotifierHostRegistered",
                    ) => OutgoingValue::Variant(Box::new(OutgoingValue::Bool(true))),

                    (
                        "/StatusNotifierWatcher",
                        "org.kde.StatusNotifierWatcher",
                        "RegisteredStatusNotifierItems",
                    ) => OutgoingValue::Variant(Box::new(OutgoingValue::Array(
                        OutgoingCompleteType::String,
                        vec![],
                    ))),

                    _ => {
                        self.reply_err(serial, sender);
                        return true;
                    }
                };

                self.reply_ok(serial, sender, vec![value]);
            }

            _ => {
                self.reply_err(serial, sender);
            }
        }

        true
    }
}

const BUILTIN_INTERFACES: &str = r#"
<interface name="org.freedesktop.DBus.Introspectable">
    <method name="Introspect">
        <arg type="s" direction="out"/>
    </method>
</interface>

<interface name="org.freedesktop.DBus.Properties">
    <method name="Get">
        <arg name="interface_name" type="s" direction="in"/>
        <arg name="property_name" type="s" direction="in"/>
        <arg type="v" direction="out"/>
    </method>
    <method name="Set">
        <arg name="interface_name" type="s" direction="in"/>
        <arg name="property_name" type="s" direction="in"/>
        <arg name="value" type="v" direction="in"/>
    </method>
    <method name="GetAll">
        <arg name="interface_name" type="s" direction="in"/>
        <arg type="a{sv}" direction="out"/>
    </method>
    <signal name="PropertiesChanged">
        <arg name="interface_name" type="s"/>
        <arg name="changed_properties" type="a{sv}"/>
        <arg name="invalidated_properties" type="as"/>
    </signal>
</interface>

<interface name="org.freedesktop.DBus.Peer">
    <method name="Ping">
    </method>
    <method name="GetMachineId">
        <arg type="s" direction="out"/>
    </method>
</interface>
"#;

const KSNI_INTERFACE: &str = r#"
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
"#;

const XML_HEADER: &str = r#"
<!DOCTYPE node PUBLIC "-//freedesktop//DTD D-BUS Object Introspection 1.0//EN" "http://www.freedesktop.org/standards/dbus/1.0/introspect.dtd">
"#;

fn ksni_introspection_xml() -> String {
    format!(
        "
{XML_HEADER}
<node>
    {KSNI_INTERFACE}
    {BUILTIN_INTERFACES}
</node>
"
    )
}

fn root_introspection_xml() -> String {
    format!(
        r#"
{XML_HEADER}
<node>
    {BUILTIN_INTERFACES}
    <node name="StatusNotifierWatcher">
        {KSNI_INTERFACE}
        {BUILTIN_INTERFACES}
    </node>
</node>
"#
    )
}
