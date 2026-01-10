use crate::{
    Event,
    dbus::{
        BuiltinDBusMessage, DBus, Message,
        messages::org_freedesktop_dbus::{AddMatch, GetAllProperties},
    },
};
use anyhow::Result;
use mute_changed::MuteChanged;
use std::borrow::Cow;
use volume_and_muted_properties::VolumeAndMutedProperties;
use volume_changed::VolumeChanged;

mod mute_changed;
mod volume_and_muted_properties;
mod volume_changed;

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

    pub(crate) fn init(&mut self, dbus: &mut DBus) -> Result<()> {
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

    pub(crate) fn on_builtin_message(
        &mut self,
        message: &BuiltinDBusMessage,
        events: &mut Vec<Event>,
    ) -> Result<()> {
        if !self.sent_initial_event {
            return Ok(());
        }

        let BuiltinDBusMessage::PropertiesChanged(message) = message else {
            return Ok(());
        };

        if let Ok(VolumeChanged { volume }) = VolumeChanged::try_from(message) {
            events.push(Event::VolumeChanged { volume });
        }

        if let Ok(MuteChanged { muted }) = MuteChanged::try_from(message) {
            events.push(Event::MuteChanged { muted });
        }

        Ok(())
    }

    pub(crate) fn on_unknown_message(
        &mut self,
        message: &Message,
        events: &mut Vec<Event>,
    ) -> Result<()> {
        if self.get_all_properties_serial == message.reply_serial() {
            let VolumeAndMutedProperties { volume, muted } =
                VolumeAndMutedProperties::try_from(message)?;
            events.push(Event::InitialSound { volume, muted });
            self.sent_initial_event = true;
        }
        Ok(())
    }
}
