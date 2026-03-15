use crate::{
    dbus::{
        Message, Oneshot, OneshotResource, Subscription, SubscriptionResource,
        decoder::{Body, IncomingMessage, Value},
        messages::{interface_is, org_freedesktop_dbus::GetAllProperties, path_is, value_is},
    },
    sansio::DBusQueue,
};
use anyhow::{Context as _, Result};

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
    pub(crate) fn new(queue: DBusQueue) -> Self {
        Self {
            oneshot: Oneshot::new(Resource::default(), queue.clone()),
            subscription: Subscription::new(Resource::default(), queue.clone()),
        }
    }

    pub(crate) fn reset(&mut self) {
        self.subscription.reset();
        self.oneshot.reset();
    }

    pub(crate) fn init(&mut self, path: &str) {
        self.subscription
            .start("org.freedesktop.NetworkManager", path);
        self.oneshot.start(path.to_string());
    }

    pub(crate) fn on_message(
        &mut self,
        message: IncomingMessage<'_>,
    ) -> Option<SsidAndStrengthEvent> {
        None.or_else(|| self.oneshot.process(message).ok().flatten())
            .or_else(|| self.subscription.process(message))
    }
}

fn parse_ssid(ssid: Value<'_>) -> Result<String> {
    value_is!(ssid, Value::Array(ssid));
    let mut iter = ssid.iter();
    let mut bytes = vec![];
    while let Some(byte) = iter.try_next()? {
        value_is!(byte, Value::Byte(byte));
        bytes.push(byte);
    }
    let ssid = String::from_utf8_lossy(&bytes).to_string();
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

    fn try_process(&self, mut body: Body<'_>) -> Result<Self::Output> {
        let properties = body.try_next()?.context("no Properties in Body")?;
        value_is!(properties, Value::Array(properties));
        let mut iter = properties.iter();
        let mut ssid = None;
        let mut strength = None;
        while let Some(property) = iter.try_next()? {
            value_is!(property, Value::DictEntry(property));
            let (key, value) = property.key_value()?;
            value_is!(key, Value::String(key));
            value_is!(value, Value::Variant(value));
            match key {
                "Ssid" => {
                    let value = value.materialize()?;
                    ssid = Some(value)
                }
                "Strength" => {
                    let value = value.materialize()?;
                    strength = Some(value)
                }
                _ => {}
            }
        }

        let ssid = ssid.context("no Ssid")?;
        let ssid = parse_ssid(ssid)?;

        let strength = strength.context("no Strength")?;
        value_is!(strength, Value::Byte(strength));

        Ok(SsidAndStrengthEvent {
            ssid: Some(ssid),
            strength: Some(strength),
        })
    }
}

impl SubscriptionResource for Resource {
    type Output = SsidAndStrengthEvent;

    fn try_process(&self, path: &str, mut body: Body<'_>) -> Result<Self::Output> {
        path_is!(path, self.path.as_deref().context("no path")?);

        let interface = body.try_next()?.context("no Interface in Body")?;
        value_is!(interface, Value::String(interface));
        interface_is!(interface, "org.freedesktop.NetworkManager.AccessPoint");

        let attributes = body.try_next()?.context("no Attributes in Body")?;
        value_is!(attributes, Value::Array(attributes));
        let mut iter = attributes.iter();
        let mut ssid = None;
        let mut strength = None;
        while let Some(attribute) = iter.try_next()? {
            value_is!(attribute, Value::DictEntry(attribute));
            let (key, value) = attribute.key_value()?;
            value_is!(key, Value::String(key));
            value_is!(value, Value::Variant(value));

            if key == "Ssid" {
                let value = value.materialize()?;
                let value = parse_ssid(value)?;
                ssid = Some(value);
            } else if key == "Strength" {
                let value = value.materialize()?;
                value_is!(value, Value::Byte(value));
                strength = Some(value);
            }
        }

        Ok(SsidAndStrengthEvent { ssid, strength })
    }

    fn set_path(&mut self, path: String) {
        self.path = Some(path)
    }
}
