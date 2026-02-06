use crate::{
    Event,
    dbus::{DBus, Message},
    liburing::IoUring,
};
use anyhow::Result;
use name_lost_or_changed::NameLostOrNameOwnerChanged;
use status_notifier_watcher::StatusNotifierWatcher;
use std::collections::HashMap;
use tray_app::TrayApp;

mod name_lost_or_changed;
mod status_notifier_watcher;
mod status_notifier_watcher_introspection;
mod tray_app;

pub(crate) struct Tray {
    status_notifier_watcher: StatusNotifierWatcher,
    name_lost_or_changed: NameLostOrNameOwnerChanged,
    registry: HashMap<String, TrayApp>,
}

impl Tray {
    pub(crate) fn new() -> Box<Self> {
        Box::new(Self {
            status_notifier_watcher: StatusNotifierWatcher::new(),
            name_lost_or_changed: NameLostOrNameOwnerChanged::new(),
            registry: HashMap::new(),
        })
    }

    pub(crate) fn init(&mut self, dbus: &mut DBus, ring: &mut IoUring) -> Result<()> {
        self.status_notifier_watcher.init(dbus, ring)?;
        self.name_lost_or_changed.init(dbus, ring)?;
        Ok(())
    }

    pub(crate) fn on_message(
        &mut self,
        dbus: &mut DBus,
        message: &Message,
        events: &mut Vec<Event>,
        ring: &mut IoUring,
    ) -> Result<()> {
        if let Some(address) = self
            .status_notifier_watcher
            .on_message(dbus, message, ring)?
        {
            println!("Added {address}");
            let mut tray_app = TrayApp::new(address.clone());
            tray_app.init(dbus, ring)?;
            self.registry.insert(address, tray_app);
        }

        if let Some(address) = self.name_lost_or_changed.on_message(message) {
            println!("Removed {address}");
            self.registry.remove(&address);
        }

        for app in self.registry.values_mut() {
            app.on_message(message, dbus, ring)?;
        }

        Ok(())
    }
}
