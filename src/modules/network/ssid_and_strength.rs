use crate::{
    dbus::{
        DBus, Message, Oneshot, OneshotResource, Subscription, SubscriptionResource,
        messages::{
            body_is, interface_is, org_freedesktop_dbus::GetAllProperties, path_is, type_is,
            value_is,
        },
        types::{CompleteType, Value},
    },
    liburing::IoUring,
};
use anyhow::{Context as _, Result};
use std::collections::HashMap;

pub(crate) struct SsidAndStrength {
    oneshot: Oneshot<Resource>,
    subscription: Subscription<Resource>,
}

#[derive(Debug)]
pub(crate) struct SsidAndStrengthEvent {
    pub(crate) ssid: Option<String>,
    pub(crate) strength: Option<u8>,
}

impl SsidAndStrength {
    pub(crate) fn new() -> Self {
        Self {
            oneshot: Oneshot::new(Resource::default()),
            subscription: Subscription::new(Resource::default()),
        }
    }

    pub(crate) fn reset(&mut self, dbus: &mut DBus, ring: &mut IoUring) -> Result<()> {
        self.subscription.reset(dbus, ring)?;
        self.oneshot.reset();
        Ok(())
    }

    pub(crate) fn init(&mut self, dbus: &mut DBus, path: &str, ring: &mut IoUring) -> Result<()> {
        self.subscription.start(dbus, path, ring)?;
        self.oneshot.start(dbus, path.to_string(), ring)?;
        Ok(())
    }

    pub(crate) fn on_message(&mut self, message: &Message) -> Option<SsidAndStrengthEvent> {
        None.or_else(|| self.oneshot.process(message))
            .or_else(|| self.subscription.process(message))
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

#[derive(Default)]
struct Resource {
    path: Option<String>,
}
impl OneshotResource for Resource {
    type Input = String;
    type Output = SsidAndStrengthEvent;

    fn make_request(&self, path: String) -> Message<'static> {
        GetAllProperties::new(
            "org.freedesktop.NetworkManager",
            path,
            "org.freedesktop.NetworkManager.AccessPoint",
        )
        .into()
    }

    fn try_process(&self, body: &[Value]) -> Result<Self::Output> {
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
}

impl SubscriptionResource for Resource {
    type Output = SsidAndStrengthEvent;

    fn try_process(&self, path: &str, interface: &str, items: &[Value]) -> Result<Self::Output> {
        interface_is!(interface, "org.freedesktop.NetworkManager.AccessPoint");
        path_is!(path, self.path.as_deref().context("no path")?);

        let mut ssid = None;
        let mut strength = None;

        for item in items {
            value_is!(item, Value::DictEntry(key, value));
            value_is!(&**key, Value::String(key));
            value_is!(&**value, Value::Variant(value));

            if key == "Ssid" {
                let value = parse_ssid(value)?;
                ssid = Some(value);
            } else if key == "Strength" {
                value_is!(&**value, Value::Byte(value));
                strength = Some(*value);
            }
        }

        Ok(SsidAndStrengthEvent { ssid, strength })
    }

    fn set_path(&mut self, path: String) {
        self.path = Some(path)
    }
}
