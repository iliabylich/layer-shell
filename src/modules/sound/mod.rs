use crate::{
    Event,
    dbus::{
        DBus, Message, Oneshot, OneshotResource, Subscription, SubscriptionResource,
        messages::{
            body_is, interface_is, org_freedesktop_dbus::GetAllProperties, path_is, type_is,
            value_is,
        },
        types::{CompleteType, Value},
    },
};
use anyhow::{Context as _, Result};

pub(crate) struct Sound {
    oneshot: Oneshot<Resource>,
    subscription: Subscription<Resource>,
}

impl Sound {
    pub(crate) fn new() -> Box<Self> {
        Box::new(Self {
            oneshot: Oneshot::new(Resource),
            subscription: Subscription::new(Resource),
        })
    }

    pub(crate) fn init(&mut self, dbus: &mut DBus) -> Result<()> {
        self.oneshot.start(dbus, ())
    }

    pub(crate) fn on_message(
        &mut self,
        dbus: &mut DBus,
        message: &Message,
        events: &mut Vec<Event>,
    ) -> Result<()> {
        if let Some((volume, muted)) = self.oneshot.process(message) {
            events.push(Event::InitialSound { volume, muted });
            self.subscription.start(dbus, "/org/local/PipewireDBus")?;

            return Ok(());
        }

        if let Some((volume, muted)) = self.subscription.process(message) {
            if let Some(volume) = volume {
                events.push(Event::VolumeChanged { volume });
            }

            if let Some(muted) = muted {
                events.push(Event::MuteChanged { muted });
            }
        }

        Ok(())
    }
}

struct Resource;

impl OneshotResource for Resource {
    type Input = ();
    type Output = (u32, bool);

    fn make_request(&self, _input: Self::Input) -> Message<'static> {
        GetAllProperties::new(
            "org.local.PipewireDBus",
            "/org/local/PipewireDBus",
            "org.local.PipewireDBus",
        )
        .into()
    }

    fn try_process(&self, body: &[Value]) -> Result<Self::Output> {
        body_is!(body, [Value::Array(item_t, array)]);
        type_is!(item_t, CompleteType::DictEntry(key_t, value_t));
        type_is!(&**key_t, CompleteType::String);
        type_is!(&**value_t, CompleteType::Variant);

        let (volume, muted) = parse(array)?;
        let volume = volume.context("no Volume")?;
        let muted = muted.context("no Muted")?;

        Ok((volume, muted))
    }
}

impl SubscriptionResource for Resource {
    type Output = (Option<u32>, Option<bool>);

    fn try_process(&self, path: &str, interface: &str, items: &[Value]) -> Result<Self::Output> {
        path_is!(path, "/org/local/PipewireDBus");
        interface_is!(interface, "org.local.PipewireDBus");

        parse(items)
    }

    fn set_path(&mut self, _: String) {}
}

fn parse(attributes: &[Value]) -> Result<(Option<u32>, Option<bool>)> {
    let mut volume = None;
    let mut muted = None;

    for attribute in attributes {
        value_is!(attribute, Value::DictEntry(key, value));
        value_is!(&**key, Value::String(key));
        value_is!(&**value, Value::Variant(value));

        if key == "Volume" {
            value_is!(&**value, Value::UInt32(value));
            volume = Some(*value);
        }

        if key == "Muted" {
            value_is!(&**value, Value::Bool(value));
            muted = Some(*value);
        }
    }

    Ok((volume, muted))
}
