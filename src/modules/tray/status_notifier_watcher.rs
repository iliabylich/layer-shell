use crate::{
    modules::tray::service::Service,
    utils::{StringRef, StringRefExt as _, dbus::queue::SessionDBusQueue},
};
use anyhow::{Context as _, Result, ensure};
use dbus::{
    DBusError, IncomingMessage, IncomingValue, MessageType, destination_is, interface_is,
    messages::{
        EmptyMethodReturn, org_freedesktop_dbus::RequestName,
        sni_host::StatusNotifierWatcherIntrospection,
    },
    messaging::DBusEncode,
    path_is, value_is,
};

struct RequestNameOrgKdeStatusNotifierWatcher;
impl DBusEncode for RequestNameOrgKdeStatusNotifierWatcher {
    type Args<'a> = ();

    fn encode<'a>((): Self::Args<'_>, buf: &'a mut [u8]) -> Result<&'a [u8], dbus::EncodeError> {
        RequestName::encode(buf, "org.kde.StatusNotifierWatcher")
    }
}

pub(crate) struct StatusNotifierWatcher;

impl StatusNotifierWatcher {
    pub(crate) fn request_ksni_name(q: &mut SessionDBusQueue) -> Result<(), DBusError> {
        let mut buf = [0; 1_024];
        let buf = RequestNameOrgKdeStatusNotifierWatcher::encode((), &mut buf)?;
        q.push_raw(buf);
        Ok(())
    }

    fn reply_ok(serial: u32, destination: &str, q: &mut SessionDBusQueue) -> Result<(), DBusError> {
        let mut buf = [0; 1_024];
        let buf = EmptyMethodReturn::encode(&mut buf, destination, serial)?;
        q.push_raw(buf);
        Ok(())
    }

    pub(crate) fn handle_incoming_request(
        message: IncomingMessage<'_>,
        q: &mut SessionDBusQueue,
    ) -> Result<Option<Service>> {
        let mut buf = [0; 1_024];
        if let Some(reply) = StatusNotifierWatcherIntrospection::new().handle(&mut buf, message)? {
            q.push_raw(reply);
            Ok(None)
        } else if let Ok((serial, sender, req)) = KSNIRequest::parse(message) {
            match req {
                KSNIRequest::NewItem { address } => {
                    Self::reply_ok(serial, sender, q)?;
                    Ok(Some(Service::new(
                        StringRef::new(sender),
                        StringRef::new(address),
                    )))
                }
                KSNIRequest::Other => {
                    Self::reply_ok(serial, sender, q)?;
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
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
                value_is!(address, IncomingValue::String(address));

                Self::NewItem { address }
            }

            _ => Self::Other,
        };

        Ok((serial, sender, req))
    }
}
