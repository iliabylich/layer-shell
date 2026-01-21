use crate::{
    Event,
    dbus::{DBus, Message},
};
use status_notifier_watcher::StatusNotifierWatcher;

mod status_notifier_watcher;

pub(crate) struct Tray {
    status_notifier_watcher: StatusNotifierWatcher,
}

impl Tray {
    pub(crate) fn new() -> Box<Self> {
        Box::new(Self {
            status_notifier_watcher: StatusNotifierWatcher::new(),
        })
    }

    pub(crate) fn init(&mut self, dbus: &mut DBus) {
        self.status_notifier_watcher.init(dbus);
    }

    pub(crate) fn on_message(
        &mut self,
        dbus: &mut DBus,
        message: &Message,
        events: &mut Vec<Event>,
    ) {
        self.status_notifier_watcher.on_message(dbus, message);
    }
}
