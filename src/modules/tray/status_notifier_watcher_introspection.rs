use crate::dbus::{
    DBus, IntrospectibleObjectAt, IntrospectibleObjectAtRequest, Message,
    types::{CompleteType, Value},
};
use std::borrow::Cow;

pub(crate) struct StatusNotifierWatcherIntrospection {
    introspection: IntrospectibleObjectAt,
}

impl StatusNotifierWatcherIntrospection {
    pub(crate) fn new() -> Self {
        Self {
            introspection: IntrospectibleObjectAt::new("org.kde.StatusNotifierWatcher"),
        }
    }

    fn reply_ok(dbus: &mut DBus, serial: u32, destination: &str, body: Vec<Value>) {
        let mut message = Message::MethodReturn {
            serial: 0,
            reply_serial: serial,
            destination: Some(Cow::Borrowed(destination)),
            sender: None,
            unix_fds: None,
            body,
        };
        dbus.enqueue(&mut message)
    }

    fn reply_err(dbus: &mut DBus, serial: u32, destination: &str) {
        let mut reply = Message::new_err_no_method(serial, destination);
        dbus.enqueue(&mut reply)
    }

    pub(crate) fn process_message(&mut self, dbus: &mut DBus, message: &Message) -> bool {
        let Ok((serial, sender, req)) = self.introspection.handle(message) else {
            return false;
        };

        match req {
            IntrospectibleObjectAtRequest::Introspect { path } => match path.as_str() {
                "/" => Self::reply_ok(
                    dbus,
                    serial,
                    &sender,
                    vec![Value::String(Cow::Owned(root_introspection_xml()))],
                ),
                "/StatusNotifierWatcher" => Self::reply_ok(
                    dbus,
                    serial,
                    &sender,
                    vec![Value::String(Cow::Owned(ksni_introspection_xml()))],
                ),
                _ => Self::reply_err(dbus, serial, &sender),
            },

            IntrospectibleObjectAtRequest::GetAllProperties { path, interface } => {
                match (path.as_str(), interface.as_str()) {
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
                        Self::reply_ok(dbus, serial, &sender, body);
                    }

                    _ => Self::reply_err(dbus, serial, &sender),
                }
            }

            IntrospectibleObjectAtRequest::GetProperty {
                path,
                interface,
                property_name,
            } => {
                let value = match (path.as_str(), interface.as_str(), property_name.as_str()) {
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
                        Self::reply_err(dbus, serial, &sender);
                        return true;
                    }
                };

                Self::reply_ok(dbus, serial, &sender, vec![value]);
            }

            _ => {
                Self::reply_err(dbus, serial, &sender);
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
