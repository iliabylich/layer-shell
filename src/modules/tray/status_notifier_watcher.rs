use crate::{
    dbus::{
        OutgoingMessage,
        decoder::{IncomingMessage, MessageType, Value},
        messages::{
            destination_is, interface_is, org_freedesktop_dbus::RequestName, path_is, value_is,
        },
    },
    ffi::ShortString,
    modules::tray::{
        service::Service, status_notifier_watcher_introspection::StatusNotifierWatcherIntrospection,
    },
    sansio::DBusQueue,
};
use anyhow::{Context as _, Result, ensure};

pub(crate) struct StatusNotifierWatcher {
    reply_serial: Option<u32>,
    introspection: StatusNotifierWatcherIntrospection,
    queue: DBusQueue,
}

impl StatusNotifierWatcher {
    pub(crate) fn new(queue: DBusQueue) -> Self {
        Self {
            reply_serial: None,
            introspection: StatusNotifierWatcherIntrospection::new(queue.copy()),
            queue,
        }
    }

    pub(crate) fn request(&mut self) {
        let mut message: OutgoingMessage =
            RequestName::new(ShortString::from("org.kde.StatusNotifierWatcher")).into();
        self.queue.push_back(&mut message);
        self.reply_serial = Some(message.serial());
    }

    fn reply_ok(&self, serial: u32, destination: &str) {
        let mut reply = OutgoingMessage::new_method_return_no_body(serial, destination);
        self.queue.push_back(&mut reply);
    }

    pub(crate) fn init(&mut self) {
        self.request()
    }

    pub(crate) fn on_message(&mut self, message: IncomingMessage<'_>) -> Option<Service> {
        if self.introspection.process_message(message) {
            return None;
        }

        if let Ok((serial, sender, req)) = KSNIRequest::parse(message) {
            match req {
                KSNIRequest::NewItem { address } => {
                    self.reply_ok(serial, sender);
                    return Some(Service::new(
                        ShortString::from(sender),
                        ShortString::from(address),
                    ));
                }
                KSNIRequest::Other => {
                    self.reply_ok(serial, sender);
                    return None;
                }
            }
        }

        None
    }
}

enum KSNIRequest<'a> {
    NewItem { address: &'a str },
    Other,
}

impl<'a> KSNIRequest<'a> {
    fn parse(message: IncomingMessage<'a>) -> Result<(u32, &'a str, Self)> {
        ensure!(message.message_type == MessageType::MethodCall);

        let serial = message.serial;
        let path = message.path.context("no Path")?;
        let member = message.member.context("no Member")?;
        let interface = message.interface.context("no Interface")?;
        let destination = message.destination.context("no Destination")?;
        let sender = message.sender.context("no Sender")?;
        let mut body = message.body.context("no Body")?;

        path_is!(path, "/StatusNotifierWatcher");
        interface_is!(interface, "org.kde.StatusNotifierWatcher");
        destination_is!(destination, "org.kde.StatusNotifierWatcher");

        let req = match member {
            "RegisterStatusNotifierItem" => {
                let address = body.try_next()?.context("no Address")?;
                value_is!(address, Value::String(address));

                Self::NewItem { address }
            }

            _ => Self::Other,
        };

        Ok((serial, sender, req))
    }
}
