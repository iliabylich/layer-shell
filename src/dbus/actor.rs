use crate::{
    Event,
    dbus::{DBus, KnownDBusMessage, Message},
    timerfd::Tick,
};
use anyhow::Result;

pub(crate) trait DBusActor {
    fn init(&mut self, dbus: &mut DBus) -> Result<()>;

    fn on_message(
        &mut self,
        message: &KnownDBusMessage,
        events: &mut Vec<Event>,
        dbus: &mut DBus,
    ) -> Result<()>;

    fn on_unknown_message(
        &mut self,
        _message: &Message,
        _events: &mut Vec<Event>,
        _dbus: &mut DBus,
    ) -> Result<()> {
        Ok(())
    }

    fn on_tick(&mut self, tick: Tick) -> Result<()>;
}
