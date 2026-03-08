use crate::{
    dbus::{
        Message,
        messages::{
            body_is, destination_is, interface_is, message_is, org_freedesktop_dbus::RequestName,
            path_is, value_is,
        },
        types::Value,
    },
    modules::tray::{
        service::Service, status_notifier_watcher_introspection::StatusNotifierWatcherIntrospection,
    },
    sansio::DBusQueue,
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

    pub(crate) fn request(&mut self, queue: &DBusQueue) {
        let mut message: Message = RequestName::new("org.kde.StatusNotifierWatcher").into();
        queue.push_back(&mut message);
        self.reply_serial = Some(message.serial());
    }

    fn reply_ok(queue: &DBusQueue, serial: u32, destination: &str) {
        let mut reply = Message::new_method_return_no_body(serial, destination);
        queue.push_back(&mut reply);
    }

    pub(crate) fn init(&mut self, queue: &DBusQueue) {
        self.request(queue)
    }

    pub(crate) fn on_message(&mut self, queue: &DBusQueue, message: &Message) -> Option<Service> {
        if self.introspection.process_message(queue, message) {
            return None;
        }

        if let Ok((serial, sender, req)) = KSNIRequest::parse(message) {
            match req {
                KSNIRequest::NewItem { address } => {
                    Self::reply_ok(queue, serial, &sender);
                    return Some(Service::new(sender, address));
                }
                KSNIRequest::Other => {
                    Self::reply_ok(queue, serial, &sender);
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
