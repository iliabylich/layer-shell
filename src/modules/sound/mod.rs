use crate::{
    Event,
    dbus::{
        Oneshot, OneshotResource, OutgoingMessage, Subscription, SubscriptionResource,
        decoder::{ArrayValue, Body, IncomingMessage, Value},
        messages::{interface_is, org_freedesktop_dbus::GetAllProperties, path_is, value_is},
    },
    event_queue::EventQueue,
    ffi::ShortString,
    sansio::DBusConnectionKind,
};
use anyhow::{Context as _, Result};

pub(crate) struct Sound {
    oneshot: Oneshot<Resource>,
    subscription: Subscription<Resource>,
    healthy: bool,
    events: EventQueue,
}

impl Sound {
    pub(crate) fn new(events: EventQueue) -> Self {
        Self {
            oneshot: Oneshot::new(Resource, DBusConnectionKind::Session),
            subscription: Subscription::new(Resource, DBusConnectionKind::Session),
            healthy: true,
            events,
        }
    }

    pub(crate) fn init(&mut self) {
        self.oneshot.send(())
    }

    pub(crate) fn on_message(&mut self, message: IncomingMessage<'_>) {
        match self.oneshot.try_rev(message) {
            Ok(Some((volume, muted))) => {
                self.events.push_back(Event::InitialSound { volume, muted });
                self.subscription.start(
                    ShortString::new_const("org.local.PipewireDBus"),
                    ShortString::new_const("/org/local/PipewireDBus"),
                );

                return;
            }
            Ok(None) => {}
            Err(err) => {
                log::error!("{err:?}");
                self.subscription.reset();
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
            self.oneshot = Oneshot::new(Resource, DBusConnectionKind::Session);
            self.oneshot.send(());
        }
    }
}

struct Resource;

impl OneshotResource for Resource {
    type Input = ();
    type Output = (u32, bool);

    fn request(&self, _input: Self::Input) -> impl Into<OutgoingMessage> {
        GetAllProperties::new(
            ShortString::new_const("org.local.PipewireDBus"),
            ShortString::new_const("/org/local/PipewireDBus"),
            ShortString::new_const("org.local.PipewireDBus"),
        )
    }

    fn try_recv(&self, mut body: Body<'_>) -> Result<Self::Output> {
        let attributes = body.try_next()?.context("expected 1 value")?;
        value_is!(attributes, Value::Array(attributes));

        let (volume, muted) = parse(attributes)?;
        let volume = volume.context("no Volume")?;
        let muted = muted.context("no Muted")?;

        Ok((volume, muted))
    }
}

impl SubscriptionResource for Resource {
    type Output = (Option<u32>, Option<bool>);

    fn try_process(&self, path: ShortString, mut body: Body<'_>) -> Result<Self::Output> {
        path_is!(path, "/org/local/PipewireDBus");

        let interface = body.try_next()?.context("no interface in Body")?;
        value_is!(interface, Value::String(interface));
        interface_is!(interface, "org.local.PipewireDBus");

        let attributes = body.try_next()?.context("no attributes in Body")?;
        value_is!(attributes, Value::Array(attributes));

        parse(attributes)
    }

    fn set_path(&mut self, _: ShortString) {}
}

fn parse(attributes: ArrayValue) -> Result<(Option<u32>, Option<bool>)> {
    let mut volume = None;
    let mut muted = None;

    let mut iter = attributes.iter();
    while let Some(item) = iter.try_next()? {
        value_is!(item, Value::DictEntry(dict_entry));

        let (key, value) = dict_entry.key_value()?;
        value_is!(key, Value::String(key));
        value_is!(value, Value::Variant(value));

        if key == "Volume" {
            let value = value.materialize()?;
            value_is!(value, Value::UInt32(value));
            volume = Some(normalize_volume(value));
        }

        if key == "Muted" {
            let value = value.materialize()?;
            value_is!(value, Value::Bool(value));
            muted = Some(value)
        }
    }

    Ok((volume, muted))
}

fn normalize_volume(volume: u32) -> u32 {
    if volume == 99 { 100 } else { volume }
}
