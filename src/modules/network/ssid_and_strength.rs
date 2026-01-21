use crate::dbus::{
    DBus, Message,
    messages::{
        body_is, interface_is, message_is,
        org_freedesktop_dbus::{AddMatch, GetAllProperties, PropertiesChanged, RemoveMatch},
        path_is, type_is, value_is,
    },
    types::{CompleteType, Value},
};
use anyhow::{Context as _, Result, ensure};
use std::collections::HashMap;

pub(crate) struct SsidAndStrength {
    path: Option<String>,
    reply_serial: Option<u32>,
}

#[derive(Debug)]
pub(crate) struct SsidAndStrengthEvent {
    pub(crate) ssid: Option<String>,
    pub(crate) strength: Option<u8>,
}

impl SsidAndStrength {
    pub(crate) fn new() -> Self {
        Self {
            path: None,
            reply_serial: None,
        }
    }

    fn unsubscribe(&mut self, dbus: &mut DBus) {
        let Some(path) = self.path.take() else {
            return;
        };

        let mut message: Message = RemoveMatch::new(&path).into();
        dbus.enqueue(&mut message);
    }

    fn subscribe(&mut self, dbus: &mut DBus, path: &str) {
        let mut message: Message = AddMatch::new(path).into();
        dbus.enqueue(&mut message);
        self.path = Some(path.to_string());
    }

    fn request(&mut self, dbus: &mut DBus, path: &str) {
        let mut message: Message = GetAllProperties::new(
            "org.freedesktop.NetworkManager",
            path,
            "org.freedesktop.NetworkManager.AccessPoint",
        )
        .into();
        dbus.enqueue(&mut message);
        self.reply_serial = Some(message.serial())
    }

    pub(crate) fn reset(&mut self, dbus: &mut DBus) {
        self.unsubscribe(dbus);
    }

    pub(crate) fn init(&mut self, dbus: &mut DBus, path: &str) {
        self.unsubscribe(dbus);
        self.subscribe(dbus, path);
        self.request(dbus, path);
    }

    fn try_parse_reply(&self, message: &Message) -> Result<SsidAndStrengthEvent> {
        ensure!(message.reply_serial() == self.reply_serial);
        message_is!(message, Message::MethodReturn { body, .. });

        body_is!(body, [Value::Array(item_t, array)]);
        type_is!(item_t, CompleteType::DictEntry(key_t, value_t));
        type_is!(&**key_t, CompleteType::String);
        type_is!(&**value_t, CompleteType::Variant);

        let mut map = HashMap::new();
        for item in array {
            value_is!(item, Value::DictEntry(key, value));
            value_is!(&**key, Value::String(key));
            value_is!(&**value, Value::Variant(value));
            map.insert(key.as_ref(), &**value);
        }

        let ssid = map.remove("Ssid").context("no Ssid")?;
        let ssid = parse_ssid(ssid)?;

        let strength = map.remove("Strength").context("no Strength")?;
        value_is!(strength, Value::Byte(strength));

        Ok(SsidAndStrengthEvent {
            ssid: Some(ssid),
            strength: Some(*strength),
        })
    }

    fn try_parse_signal(&self, message: &Message) -> Result<SsidAndStrengthEvent> {
        let expected_path = self.path.as_ref().context("not subscribed")?.as_str();

        let PropertiesChanged {
            path,
            interface,
            mut changes,
        } = PropertiesChanged::try_from(message)?;

        path_is!(path, expected_path);
        interface_is!(interface, "org.freedesktop.NetworkManager.AccessPoint");

        let ssid = if let Some(ssid) = changes.remove("Ssid") {
            Some(parse_ssid(&ssid)?)
        } else {
            None
        };

        let strength = if let Some(strength) = changes.remove("Strength") {
            value_is!(strength, Value::Byte(strength));
            Some(strength)
        } else {
            None
        };

        Ok(SsidAndStrengthEvent { ssid, strength })
    }

    pub(crate) fn on_message(&mut self, message: &Message) -> Option<SsidAndStrengthEvent> {
        if let Ok(e) = self.try_parse_reply(message) {
            return Some(e);
        }

        if let Ok(e) = self.try_parse_signal(message) {
            return Some(e);
        }

        None
    }
}

fn parse_ssid(ssid: &Value) -> Result<String> {
    value_is!(ssid, Value::Array(CompleteType::Byte, ssid));
    let ssid = ssid
        .iter()
        .map(|byte| {
            value_is!(byte, Value::Byte(byte));
            Ok(*byte)
        })
        .collect::<Result<Vec<_>>>()?;
    let ssid = String::from_utf8_lossy(&ssid).to_string();
    Ok(ssid)
}
