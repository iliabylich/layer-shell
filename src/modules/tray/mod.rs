use crate::{
    Event, dbus::decoder::IncomingMessage, event_queue::EventQueue, ffi::ShortString,
    modules::tray::app::TrayEvent, sansio::DBusQueue,
};
use app::App;
pub use icon::{TrayIcon, TrayIconPixmap};
pub use item::TrayItem;
use name_lost_or_changed::NameLostOrNameOwnerChanged;
use service::Service;
use status_notifier_watcher::StatusNotifierWatcher;
use std::collections::HashMap;
use uuid::UUID;

mod app;
mod icon;
mod item;
mod name_lost_or_changed;
mod service;
mod status_notifier_watcher;
mod status_notifier_watcher_introspection;
mod uuid;

pub(crate) struct Tray {
    status_notifier_watcher: StatusNotifierWatcher,
    name_lost_or_changed: NameLostOrNameOwnerChanged,
    registry: HashMap<Service, App>,
    events: EventQueue,
    queue: DBusQueue,
}

impl Tray {
    pub(crate) fn new(events: EventQueue, queue: DBusQueue) -> Self {
        Self {
            status_notifier_watcher: StatusNotifierWatcher::new(queue.copy()),
            name_lost_or_changed: NameLostOrNameOwnerChanged::new(queue.copy()),
            registry: HashMap::new(),
            events,
            queue,
        }
    }

    pub(crate) fn init(&mut self) {
        self.status_notifier_watcher.init();
        self.name_lost_or_changed.init();
    }

    pub(crate) fn on_message(&mut self, message: IncomingMessage<'_>) {
        if let Some(service) = self.status_notifier_watcher.on_message(message) {
            log::info!(target: "Tray", "Added {service:?}");
            let mut tray_app = App::new(service, self.queue.copy());
            tray_app.init();
            self.registry.insert(service, tray_app);
            return;
        }

        if let Some(service) = self.name_lost_or_changed.on_message(message) {
            let Some(key) = self
                .registry
                .keys()
                .find(|s| s.name() == service || s.raw_address() == service)
                .cloned()
            else {
                return;
            };

            let Some(mut tray_app) = self.registry.remove(&key) else {
                return;
            };

            log::info!(target: "Tray", "Removed {service}");
            tray_app.reset();
            self.events.push_back(Event::TrayAppRemoved { service })
        }

        for (service, app) in &mut self.registry {
            if let Some(event) = app.on_message(message) {
                let service = service.name();

                let event = match event {
                    TrayEvent::Initialized(icon, layout) => Event::TrayAppAdded {
                        service,
                        items: layout.into(),
                        icon,
                    },
                    TrayEvent::IconUpdated(icon) => Event::TrayAppIconUpdated { service, icon },
                    TrayEvent::MenuUpdated(layout) => Event::TrayAppMenuUpdated {
                        service,
                        items: layout.into(),
                    },
                };
                self.events.push_back(event);
            }
        }
    }

    pub(crate) fn trigger(&self, uuid: ShortString) {
        let Ok((service, id)) = UUID::decode(uuid) else {
            log::error!("malformed UUID: {uuid:?}");
            return;
        };

        let Some(key) = self
            .registry
            .keys()
            .find(|k| k.name() == service || k.raw_address() == service)
            .cloned()
        else {
            log::info!(target: "Tray", "service {service} doesn't exist");
            return;
        };

        let Some(tray_app) = self.registry.get(&key) else {
            log::info!(target: "Tray", "service {service} doesn't exist");
            return;
        };

        tray_app.trigger(id);
    }
}
