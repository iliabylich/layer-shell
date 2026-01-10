use crate::dbus::{
    BuiltinDBusMessage, DBus, Message, messages::org_freedesktop_dbus::RequestName, types::Value,
};
use anyhow::Result;
use introspect_request::IntrospectRequest;
use introspect_response::IntrospectResponse;
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

    pub(crate) fn init(&mut self, dbus: &mut DBus) -> Result<()> {
        let mut message: Message =
            RequestName::new(Cow::Borrowed("org.me.LayerShellTmpControl")).into();
        dbus.enqueue(&mut message)?;

        Ok(())
    }

    pub(crate) fn on_builtin_message(
        &mut self,
        message: &BuiltinDBusMessage,
        dbus: &mut DBus,
    ) -> Result<()> {
        let BuiltinDBusMessage::IntrospectRequest(req) = message else {
            return Ok(());
        };

        let Ok(_) = IntrospectRequest::try_from(req) else {
            return Ok(());
        };

        let mut reply: Message = IntrospectResponse::new(req.serial, &req.sender).into();
        dbus.enqueue(&mut reply)?;

        Ok(())
    }

    pub(crate) fn on_unknown_message(
        &mut self,
        message: &Message,
        dbus: &mut DBus,
    ) -> Result<Option<ControlRequest>> {
        let Ok(req) = AnyControlRequest::try_from(message) else {
            return Ok(None);
        };

        let AnyControlRequest::Known(req) = req else {
            let mut reply = Message::Error {
                serial: 0,
                error_name: String::from("org.freedesktop.DBus.Error.UnknownMethod"),
                reply_serial: message.serial(),
                destination: message.sender().map(|s| Cow::Owned(s.to_string())),
                sender: None,
                unix_fds: None,
                body: vec![Value::String(String::from("Unknown method"))],
            };
            dbus.enqueue(&mut reply)?;
            return Ok(None);
        };

        let mut reply = Message::MethodReturn {
            serial: 0,
            reply_serial: message.serial(),
            destination: message.sender().map(|s| Cow::Owned(s.to_string())),
            sender: None,
            unix_fds: None,
            body: vec![],
        };
        dbus.enqueue(&mut reply)?;

        Ok(Some(req))
    }
}
