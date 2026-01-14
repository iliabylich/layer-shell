use crate::dbus::{DBus, Message, messages::org_freedesktop_dbus::RequestName, types::Value};
use introspect_request::ControlIntrospectRequest;
use introspect_response::ControlIntrospectResponse;
use request::AnyControlRequest;
pub(crate) use request::ControlRequest;
use std::borrow::Cow;

mod introspect_request;
mod introspect_response;
mod request;

pub(crate) struct Control;

impl Control {
    pub(crate) fn new() -> Box<Self> {
        Box::new(Self)
    }

    pub(crate) fn init(&mut self, dbus: &mut DBus) {
        let mut message: Message =
            RequestName::new(Cow::Borrowed("org.me.LayerShellControl")).into();
        dbus.enqueue(&mut message);
    }

    fn reply_ok(dbus: &mut DBus, message: &Message) {
        let mut reply = Message::MethodReturn {
            serial: 0,
            reply_serial: message.serial(),
            destination: message.sender().map(|s| Cow::Owned(s.to_string())),
            sender: None,
            unix_fds: None,
            body: vec![],
        };
        dbus.enqueue(&mut reply);
    }

    fn reply_err(dbus: &mut DBus, message: &Message) {
        let mut reply = Message::Error {
            serial: 0,
            error_name: String::from("org.freedesktop.DBus.Error.UnknownMethod"),
            reply_serial: message.serial(),
            destination: message.sender().map(|s| Cow::Owned(s.to_string())),
            sender: None,
            unix_fds: None,
            body: vec![Value::String(String::from("Unknown method"))],
        };
        dbus.enqueue(&mut reply);
    }

    pub(crate) fn on_message(
        &mut self,
        message: &Message,
        dbus: &mut DBus,
    ) -> Option<ControlRequest> {
        if let Ok(ControlIntrospectRequest { sender, serial }) =
            ControlIntrospectRequest::try_from(message)
        {
            let mut reply: Message = ControlIntrospectResponse::new(serial, &sender).into();
            dbus.enqueue(&mut reply);
            return None;
        }

        if let Ok(req) = AnyControlRequest::try_from(message) {
            match req {
                AnyControlRequest::Known(req) => {
                    Self::reply_ok(dbus, message);
                    return Some(req);
                }

                AnyControlRequest::Unknown => {
                    Self::reply_err(dbus, message);
                    return None;
                }
            }
        }

        None
    }
}
