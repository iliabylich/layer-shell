use crate::{
    Event,
    dbus::{
        Oneshot, OneshotResource, OutgoingMessage, Subscription, SubscriptionResource,
        decoder::{ArrayValue, Body, IncomingMessage, Value},
        messages::{interface_is, org_freedesktop_dbus::GetAllProperties, value_is},
    },
    event_queue::EventQueue,
    ffi::ShortString,
    sansio::DBusQueue,
};
use anyhow::{Context as _, Result, ensure};

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
            oneshot: Oneshot::new(Resource, queue.copy()),
            subscription: Subscription::new(Resource, queue.copy()),
            healthy: true,
            events,
            queue,
        }
    }

    pub(crate) fn init(&mut self) {
        self.oneshot.start(())
    }

    pub(crate) fn on_message(&mut self, message: IncomingMessage<'_>) {
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
            self.oneshot = Oneshot::new(Resource, self.queue.copy());
            self.oneshot.start(());
        }
    }
}

struct Resource;

impl OneshotResource for Resource {
    type Input = ();
    type Output = (u32, bool);

    fn make_request(&self, _input: Self::Input) -> OutgoingMessage<'static> {
        GetAllProperties::new(
            ShortString::from("org.local.PipewireDBus"),
            ShortString::from("/org/local/PipewireDBus"),
            "org.local.PipewireDBus",
        )
        .into()
    }

    fn try_process(&self, mut body: Body<'_>) -> Result<Self::Output> {
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

    fn try_process(&self, path: &str, mut body: Body<'_>) -> Result<Self::Output> {
        ensure!(path == "/org/local/PipewireDBus");

        let interface = body.try_next()?.context("no interface in Body")?;
        value_is!(interface, Value::String(interface));
        interface_is!(interface, "org.local.PipewireDBus");

        let attributes = body.try_next()?.context("no attributes in Body")?;
        value_is!(attributes, Value::Array(attributes));

        parse(attributes)
    }

    fn set_path(&mut self, _: String) {}
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
