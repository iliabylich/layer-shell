use crate::{
    dbus::{
        IntrospectibleObjectAt, IntrospectibleObjectAtRequest, OutgoingMessage,
        decoder::IncomingMessage,
        types::{CompleteType, Value},
    },
    ffi::ShortString,
    sansio::DBusQueue,
};
use std::borrow::Cow;

pub(crate) struct StatusNotifierWatcherIntrospection {
    introspection: IntrospectibleObjectAt,
    queue: DBusQueue,
}

impl StatusNotifierWatcherIntrospection {
    pub(crate) fn new(queue: DBusQueue) -> Self {
        Self {
            introspection: IntrospectibleObjectAt::new("org.kde.StatusNotifierWatcher"),
            queue,
        }
    }

    fn reply_ok(&self, serial: u32, destination: &str, body: Vec<Value>) {
        let mut message = OutgoingMessage::MethodReturn {
            serial: 0,
            reply_serial: serial,
            destination: Some(ShortString::from(destination)),
            sender: None,
            unix_fds: None,
            body,
        };
        self.queue.push_back(&mut message)
    }

    fn reply_err(&self, serial: u32, destination: &str) {
        let mut reply = OutgoingMessage::new_err_no_method(serial, destination);
        self.queue.push_back(&mut reply)
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
                    vec![Value::String(Cow::Owned(root_introspection_xml()))],
                ),
                "/StatusNotifierWatcher" => self.reply_ok(
                    serial,
                    sender,
                    vec![Value::String(Cow::Owned(ksni_introspection_xml()))],
                ),
                _ => self.reply_err(serial, sender),
            },

            IntrospectibleObjectAtRequest::GetAllProperties { path, interface } => {
                match (path, interface) {
                    ("/StatusNotifierWatcher", "org.kde.StatusNotifierWatcher") => {
                        let body = vec![Value::Array(
                            CompleteType::DictEntry(
                                Box::new(CompleteType::String),
                                Box::new(CompleteType::Variant),
                            ),
                            vec![
                                Value::DictEntry(
                                    Box::new(Value::String(Cow::Borrowed("ProtocolVersion"))),
                                    Box::new(Value::Variant(Box::new(Value::Int32(42)))),
                                ),
                                Value::DictEntry(
                                    Box::new(Value::String(Cow::Borrowed(
                                        "IsStatusNotifierHostRegistered",
                                    ))),
                                    Box::new(Value::Variant(Box::new(Value::Bool(true)))),
                                ),
                                Value::DictEntry(
                                    Box::new(Value::String(Cow::Borrowed(
                                        "RegisteredStatusNotifierItems",
                                    ))),
                                    Box::new(Value::Variant(Box::new(Value::Array(
                                        CompleteType::String,
                                        vec![],
                                    )))),
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
                    ) => Value::Variant(Box::new(Value::Int32(42))),

                    (
                        "/StatusNotifierWatcher",
                        "org.kde.StatusNotifierWatcher",
                        "IsStatusNotifierHostRegistered",
                    ) => Value::Variant(Box::new(Value::Bool(true))),

                    (
                        "/StatusNotifierWatcher",
                        "org.kde.StatusNotifierWatcher",
                        "RegisteredStatusNotifierItems",
                    ) => Value::Variant(Box::new(Value::Array(CompleteType::String, vec![]))),

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
