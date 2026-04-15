use crate::{Event, event_queue::EventQueue, modules::SessionDBus};
use anyhow::{Context as _, Result};
use mini_sansio_dbus::{
    IncomingArrayValue, IncomingMessage, IncomingValue, MethodCall, Subscription, interface_is,
    messages::org_freedesktop_dbus::GetAllProperties, path_is, value_is,
};

pub(crate) struct Sound {
    oneshot: MethodCall<(), (u32, bool), ()>,
    subscription: Subscription<(Option<u32>, Option<bool>)>,
    healthy: bool,
}

impl Sound {
    pub(crate) fn new() -> Self {
        Self {
            oneshot: GET,
            subscription: SUBSCRIPTION,
            healthy: true,
        }
    }

    pub(crate) fn init(&mut self) {
        self.oneshot.send((), SessionDBus::queue())
    }

    pub(crate) fn on_message(&mut self, message: IncomingMessage<'_>) {
        match self.oneshot.try_recv(message) {
            Ok(Some((volume, muted))) => {
                EventQueue::push_back(Event::InitialSound { volume, muted });
                self.subscription.start(
                    "org.local.PipewireDBus",
                    "/org/local/PipewireDBus",
                    SessionDBus::queue(),
                );

                return;
            }
            Ok(None) => {}
            Err(err) => {
                log::error!("{err:?}");
                self.subscription.reset(SessionDBus::queue());
                self.healthy = false;
                return;
            }
        }

        if let Some((volume, muted)) = self.subscription.process(message) {
            if let Some(volume) = volume {
                EventQueue::push_back(Event::VolumeChanged { volume });
            }

            if let Some(muted) = muted {
                EventQueue::push_back(Event::MuteChanged { muted });
            }
        }
    }

    pub(crate) fn tick(&mut self, tick: u64) {
        if !self.healthy && tick.is_multiple_of(2) {
            self.healthy = true;
            self.oneshot.reset();
            self.oneshot.send((), SessionDBus::queue());
        }
    }
}

const GET: MethodCall<(), (u32, bool), ()> = MethodCall::builder()
    .send(&|_input, _data| {
        GetAllProperties::build(
            "org.local.PipewireDBus",
            "/org/local/PipewireDBus",
            "org.local.PipewireDBus",
        )
    })
    .try_process(&|mut body, _data| {
        let attributes = body.try_next()?.context("expected 1 value")?;
        value_is!(attributes, IncomingValue::Array(attributes));

        let (volume, muted) = parse(attributes)?;
        let volume = volume.context("no Volume")?;
        let muted = muted.context("no Muted")?;

        Ok((volume, muted))
    });

const SUBSCRIPTION: Subscription<(Option<u32>, Option<bool>)> =
    Subscription::new(&|mut body, path, _subscribed_to| {
        path_is!(path, "/org/local/PipewireDBus");

        let interface = body.try_next()?.context("no interface in Body")?;
        value_is!(interface, IncomingValue::String(interface));
        interface_is!(interface, "org.local.PipewireDBus");

        let attributes = body.try_next()?.context("no attributes in Body")?;
        value_is!(attributes, IncomingValue::Array(attributes));

        parse(attributes).map_err(|err| err.into())
    });

fn parse(attributes: IncomingArrayValue) -> Result<(Option<u32>, Option<bool>)> {
    let mut volume = None;
    let mut muted = None;

    let mut iter = attributes.iter();
    while let Some(item) = iter.try_next()? {
        value_is!(item, IncomingValue::DictEntry(dict_entry));

        let (key, value) = dict_entry.key_value()?;
        value_is!(key, IncomingValue::String(key));
        value_is!(value, IncomingValue::Variant(value));

        if key == "Volume" {
            let value = value.materialize()?;
            value_is!(value, IncomingValue::UInt32(mut value));
            if value == 99 {
                value = 100;
            }
            volume = Some(value);
        }

        if key == "Muted" {
            let value = value.materialize()?;
            value_is!(value, IncomingValue::Bool(value));
            muted = Some(value)
        }
    }

    Ok((volume, muted))
}
