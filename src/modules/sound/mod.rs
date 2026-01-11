use crate::{
    Event,
    dbus::{
        DBus, Message,
        messages::{
            body_is, interface_is, message_is,
            org_freedesktop_dbus::{AddMatch, GetAllProperties, PropertiesChanged},
            path_is, type_is, value_is,
        },
        types::{CompleteType, Value},
    },
};
use anyhow::{Context as _, Result, ensure};
use std::{borrow::Cow, collections::HashMap};

pub(crate) struct Sound {
    sent_initial_event: bool,
    reply_serial: Option<u32>,
}

impl Sound {
    pub(crate) fn new() -> Box<Self> {
        Box::new(Self {
            sent_initial_event: false,
            reply_serial: None,
        })
    }

    pub(crate) fn init(&mut self, dbus: &mut DBus) {
        let mut message: Message = GetAllProperties::new(
            "org.local.PipewireDBus",
            "/org/local/PipewireDBus",
            "org.local.PipewireDBus",
        )
        .into();
        dbus.enqueue(&mut message);
        self.reply_serial = Some(message.serial());

        let mut message: Message = AddMatch::new("/org/local/PipewireDBus").into();
        dbus.enqueue(&mut message);
    }

    fn try_parse_reply(&self, message: &Message) -> Result<(u32, bool)> {
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
            map.insert(Cow::Borrowed(key.as_str()), &**value);
        }

        let volume = map.remove("Volume").context("no Volume")?;
        let muted = map.remove("Muted").context("no Muted")?;

        value_is!(*volume, Value::UInt32(volume));
        value_is!(*muted, Value::Bool(muted));

        Ok((volume, muted))
    }

    fn try_parse_signal(&self, message: &Message) -> Result<(Option<u32>, Option<bool>)> {
        let PropertiesChanged {
            path,
            interface,
            changes,
        } = PropertiesChanged::try_from(message)?;

        path_is!(path, "/org/local/PipewireDBus");
        interface_is!(interface, "org.local.PipewireDBus");

        let volume = if let Some(volume) = changes.get("Volume") {
            value_is!(volume, Value::UInt32(volume));
            Some(*volume)
        } else {
            None
        };

        let muted = if let Some(muted) = changes.get("Muted") {
            value_is!(muted, Value::Bool(muted));
            Some(*muted)
        } else {
            None
        };

        Ok((volume, muted))
    }

    pub(crate) fn on_message(&mut self, message: &Message, events: &mut Vec<Event>) {
        if let Ok((volume, muted)) = self.try_parse_reply(message) {
            events.push(Event::InitialSound { volume, muted });
            self.sent_initial_event = true;
            return;
        }

        if self.sent_initial_event
            && let Ok((volume, muted)) = self.try_parse_signal(message)
        {
            if let Some(volume) = volume {
                events.push(Event::VolumeChanged { volume });
            }

            if let Some(muted) = muted {
                events.push(Event::MuteChanged { muted });
            }
        }
    }
}
