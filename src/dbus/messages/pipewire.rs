use super::{body_is, message_is, org_freedesktop_dbus::PropertiesChanged, type_is, value_is};
use crate::dbus::types::{CompleteType, Message, Value};
use anyhow::{Context as _, Result, ensure};
use std::{borrow::Cow, collections::HashMap};

#[derive(Debug)]
pub(crate) struct VolumeChanged {
    pub(crate) volume: u32,
}

impl TryFrom<&Message> for VolumeChanged {
    type Error = anyhow::Error;

    fn try_from(message: &Message) -> Result<Self> {
        let PropertiesChanged {
            path,
            interface,
            mut changes,
        } = PropertiesChanged::try_from(message)?;

        ensure!(path == "/org/local/PipewireDBus");
        ensure!(interface == "org.local.PipewireDBus");

        let volume = changes.remove("Volume").context("unrelaterd")?;
        value_is!(volume, Value::UInt32(volume));

        Ok(Self { volume })
    }
}

#[derive(Debug)]
pub(crate) struct MuteChanged {
    pub(crate) muted: bool,
}

impl TryFrom<&Message> for MuteChanged {
    type Error = anyhow::Error;

    fn try_from(message: &Message) -> Result<Self> {
        let PropertiesChanged {
            path,
            interface,
            mut changes,
        } = PropertiesChanged::try_from(message)?;

        ensure!(path == "/org/local/PipewireDBus");
        ensure!(interface == "org.local.PipewireDBus");

        let muted = changes.remove("Muted").context("unrelaterd")?;
        value_is!(muted, Value::Bool(muted));

        Ok(Self { muted })
    }
}

#[derive(Debug)]
pub(crate) struct VolumeAndMutedProperties {
    pub(crate) volume: u32,
    pub(crate) muted: bool,
}

impl TryFrom<&Message> for VolumeAndMutedProperties {
    type Error = anyhow::Error;

    fn try_from(message: &Message) -> Result<Self> {
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

        Ok(Self { volume, muted })
    }
}
