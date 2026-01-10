use crate::{
    Event,
    dbus::{
        DBus, DBusActor, KnownDBusMessage, Message,
        messages::{
            org_freedesktop_dbus::{AddMatch, GetAllProperties},
            pipewire::{MuteChanged, VolumeAndMutedProperties, VolumeChanged},
        },
    },
    timerfd::Tick,
};
use anyhow::Result;
use std::borrow::Cow;

pub(crate) struct Sound {
    sent_initial_event: bool,
    get_all_properties_serial: Option<u32>,
}

impl Sound {
    pub(crate) fn new() -> Box<Self> {
        Box::new(Self {
            sent_initial_event: false,
            get_all_properties_serial: None,
        })
    }
}

impl DBusActor for Sound {
    fn init(&mut self, dbus: &mut DBus) -> Result<()> {
        let mut message: Message = GetAllProperties::new(
            Cow::Borrowed("org.local.PipewireDBus"),
            Cow::Borrowed("/org/local/PipewireDBus"),
            Cow::Borrowed("org.local.PipewireDBus"),
        )
        .into();
        dbus.enqueue(&mut message)?;
        self.get_all_properties_serial = Some(message.serial());

        let message = AddMatch::new(Cow::Borrowed("/org/local/PipewireDBus"));
        dbus.enqueue(&mut message.into())?;

        Ok(())
    }

    fn on_message(
        &mut self,
        message: &KnownDBusMessage,
        events: &mut Vec<Event>,
        _dbus: &mut DBus,
    ) -> Result<()> {
        if !self.sent_initial_event {
            return Ok(());
        }

        match message {
            KnownDBusMessage::VolumeChanged(VolumeChanged { volume }) => {
                events.push(Event::VolumeChanged { volume: *volume });
            }
            KnownDBusMessage::MuteChanged(MuteChanged { muted }) => {
                events.push(Event::MuteChanged { muted: *muted });
            }
            _ => return Ok(()),
        };

        Ok(())
    }

    fn on_unknown_message(
        &mut self,
        message: &Message,
        events: &mut Vec<Event>,
        _dbus: &mut DBus,
    ) -> Result<()> {
        if self.get_all_properties_serial == message.reply_serial() {
            let VolumeAndMutedProperties { volume, muted } =
                VolumeAndMutedProperties::try_from(message)?;
            events.push(Event::InitialSound { volume, muted });
            self.sent_initial_event = true;
        }
        Ok(())
    }

    fn on_tick(&mut self, _tick: Tick) -> Result<()> {
        Ok(())
    }
}
