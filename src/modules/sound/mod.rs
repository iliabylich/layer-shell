use crate::{
    Event,
    dbus::{
        Message, Oneshot, OneshotResource, Subscription, SubscriptionResource,
        messages::{
            body_is, interface_is, org_freedesktop_dbus::GetAllProperties, path_is, type_is,
            value_is,
        },
        types::{CompleteType, Value},
    },
    event_queue::EventQueue,
    sansio::DBusQueue,
};
use anyhow::{Context as _, Result};

pub(crate) struct Sound {
    oneshot: Oneshot<Resource>,
    subscription: Subscription<Resource>,
    healthy: bool,
    events: EventQueue,
    queue: DBusQueue,
}

impl Sound {
    pub(crate) fn new(events: EventQueue, queue: DBusQueue) -> Self {
        Self {
            oneshot: Oneshot::new(Resource, queue.clone()),
            subscription: Subscription::new(Resource, queue.clone()),
            healthy: true,
            events,
            queue,
        }
    }

    pub(crate) fn init(&mut self) {
        self.oneshot.start(())
    }

    pub(crate) fn on_message(&mut self, message: &Message) {
        match self.oneshot.process(message) {
            Ok(Some((volume, muted))) => {
                self.events.push_back(Event::InitialSound { volume, muted });
                self.subscription
                    .start("org.local.PipewireDBus", "/org/local/PipewireDBus");

                return;
            }
            Ok(None) => {}
            Err(err) => {
                log::error!("{err:?}");
                self.healthy = false;
                return;
            }
        }

        if let Some((volume, muted)) = self.subscription.process(message) {
            if let Some(volume) = volume {
                self.events.push_back(Event::VolumeChanged { volume });
            }

            if let Some(muted) = muted {
                self.events.push_back(Event::MuteChanged { muted });
            }
        }
    }

    pub(crate) fn tick(&mut self, tick: u64) {
        if !self.healthy && tick.is_multiple_of(2) {
            self.healthy = true;
            self.oneshot = Oneshot::new(Resource, self.queue.clone());
            self.oneshot.start(());
        }
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
            volume = Some(normalize_volume(*value));
        }

        if key == "Muted" {
            value_is!(&**value, Value::Bool(value));
            muted = Some(*value);
        }
    }

    Ok((volume, muted))
}

fn normalize_volume(volume: u32) -> u32 {
    if volume == 99 { 100 } else { volume }
}
