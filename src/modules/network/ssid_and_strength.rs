use crate::{
    dbus::{
        MethodCall, Subscription,
        decoder::{ArrayValue, IncomingMessage, Value},
        messages::{interface_is, org_freedesktop_dbus::GetAllProperties, path_is, value_is},
    },
    sansio::DBusConnectionKind,
    utils::StringRef,
};
use anyhow::{Context as _, Result};

pub(crate) struct SsidAndStrength {
    oneshot: MethodCall<StringRef, SsidAndStrengthEvent, ()>,
    subscription: Subscription<SsidAndStrengthEvent>,
}

#[derive(Debug)]
pub(crate) struct SsidAndStrengthEvent {
    pub(crate) ssid: Option<StringRef>,
    pub(crate) strength: Option<u8>,
}

impl SsidAndStrength {
    pub(crate) fn new() -> Self {
        Self {
            oneshot: GET,
            subscription: SUBSCRIPTION,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.subscription.reset();
        self.oneshot.reset();
    }

    pub(crate) fn init(&mut self, path: StringRef) {
        self.subscription
            .start("org.freedesktop.NetworkManager", path.clone());
        self.oneshot.send(path);
    }

    pub(crate) fn on_message(
        &mut self,
        message: IncomingMessage<'_>,
    ) -> Option<SsidAndStrengthEvent> {
        None.or_else(|| self.oneshot.try_recv(message).ok().flatten())
            .or_else(|| self.subscription.process(message))
    }
}

const GET: MethodCall<StringRef, SsidAndStrengthEvent, ()> = MethodCall::builder()
    .send(&|path, _data| {
        GetAllProperties::build(
            "org.freedesktop.NetworkManager",
            path,
            "org.freedesktop.NetworkManager.AccessPoint",
        )
    })
    .try_process(&|mut body, _data| {
        let properties = body.try_next()?.context("no Properties in Body")?;
        value_is!(properties, Value::Array(properties));
        let (ssid, strength) = parse_properties(properties)?;

        let ssid = ssid.context("no Ssid")?;
        let strength = strength.context("no Strength")?;

        Ok(SsidAndStrengthEvent {
            ssid: Some(ssid),
            strength: Some(strength),
        })
    })
    .kind(DBusConnectionKind::System);

const SUBSCRIPTION: Subscription<SsidAndStrengthEvent> = Subscription::builder()
    .try_process(&|mut body, path, subscribed_to| {
        path_is!(path, subscribed_to);

        let interface = body.try_next()?.context("no Interface in Body")?;
        value_is!(interface, Value::String(interface));
        interface_is!(interface, "org.freedesktop.NetworkManager.AccessPoint");

        let properties = body.try_next()?.context("no Properties in Body")?;
        value_is!(properties, Value::Array(properties));
        let (ssid, strength) = parse_properties(properties)?;

        Ok(SsidAndStrengthEvent { ssid, strength })
    })
    .kind(DBusConnectionKind::System);

fn parse_properties(properties: ArrayValue<'_>) -> Result<(Option<StringRef>, Option<u8>)> {
    let mut iter = properties.iter();
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

    Ok((ssid, strength))
}

fn parse_ssid(ssid: Value<'_>) -> Result<StringRef> {
    value_is!(ssid, Value::Array(ssid));
    let mut iter = ssid.iter();
    let mut bytes = vec![];
    while let Some(byte) = iter.try_next()? {
        value_is!(byte, Value::Byte(byte));
        bytes.push(byte);
    }
    let ssid = String::from_utf8_lossy(&bytes).to_string();
    Ok(StringRef::new(ssid.as_str()))
}
