use crate::{
    dbus::{
        DBus, Message,
        messages::{
            body_is, destination_is, interface_is, message_is, org_freedesktop_dbus::RequestName,
            path_is, value_is,
        },
        types::Value,
    },
    modules::tray::status_notifier_watcher_introspection::StatusNotifierWatcherIntrospection,
};
use anyhow::Result;

pub(crate) struct StatusNotifierWatcher {
    reply_serial: Option<u32>,
    introspection: StatusNotifierWatcherIntrospection,
}

impl StatusNotifierWatcher {
    pub(crate) fn new() -> Self {
        Self {
            reply_serial: None,
            introspection: StatusNotifierWatcherIntrospection::new(),
        }
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

    pub(crate) fn init(&mut self, dbus: &mut DBus) {
        self.request(dbus);
    }

    pub(crate) fn on_message(&mut self, dbus: &mut DBus, message: &Message) -> Option<String> {
        if self.introspection.process_message(dbus, message) {
            return None;
        }

        if let Ok((serial, sender, req)) = KSNIRequest::parse(message) {
            match req {
                KSNIRequest::NewItem { address } => {
                    Self::reply_ok(dbus, serial, &sender);
                    return Some(address);
                }
                KSNIRequest::Other => {
                    Self::reply_ok(dbus, serial, &sender);
                    return None;
                }
            }
        }

        None
    }
}

enum KSNIRequest {
    NewItem { address: String },
    Other,
}

impl KSNIRequest {
    fn parse(message: &Message) -> Result<(u32, String, Self)> {
        message_is!(
            message,
            Message::MethodCall {
                serial,
                path,
                member,
                interface: Some(interface),
                destination: Some(destination),
                sender: Some(sender),
                body,
                ..
            }
        );

        path_is!(path, "/StatusNotifierWatcher");
        interface_is!(interface, "org.kde.StatusNotifierWatcher");
        destination_is!(destination, "org.kde.StatusNotifierWatcher");

        let req = match member.as_ref() {
            "RegisterStatusNotifierItem" => {
                body_is!(body, [address]);
                value_is!(address, Value::String(address));

                Self::NewItem {
                    address: address.to_string(),
                }
            }

            _ => Self::Other,
        };

        Ok((*serial, sender.to_string(), req))
    }
}
