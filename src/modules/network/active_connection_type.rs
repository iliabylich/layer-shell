use crate::dbus::{
    DBus, Message,
    messages::{body_is, message_is, org_freedesktop_dbus::GetProperty, value_is},
    types::Value,
};
use anyhow::{Result, ensure};

pub(crate) struct ActiveConnectionType {
    reply_serial: Option<u32>,
    path: Option<String>,
}

impl ActiveConnectionType {
    pub(crate) fn new() -> Self {
        Self {
            reply_serial: None,
            path: None,
        }
    }

    pub(crate) fn request(&mut self, dbus: &mut DBus, path: &str) {
        let mut message: Message = GetProperty::new(
            "org.freedesktop.NetworkManager",
            path,
            "org.freedesktop.NetworkManager.Connection.Active",
            "Type",
        )
        .into();
        dbus.enqueue(&mut message);
        self.reply_serial = Some(message.serial());
        self.path = Some(path.to_string())
    }

    pub(crate) fn reset(&mut self) {
        self.reply_serial = None;
    }

    fn try_parse_reply(&self, message: &Message) -> Result<String> {
        ensure!(self.reply_serial == message.reply_serial());
        message_is!(message, Message::MethodReturn { body, .. });
        body_is!(body, [type_]);
        value_is!(type_, Value::Variant(type_));
        value_is!(&**type_, Value::String(type_));

        Ok(type_.clone())
    }

    pub(crate) fn on_message(&mut self, message: &Message) -> Option<(bool, String)> {
        if let Ok(type_) = self.try_parse_reply(message) {
            return Some((type_.contains("wireless"), self.path.clone()?));
        }

        None
    }
}
