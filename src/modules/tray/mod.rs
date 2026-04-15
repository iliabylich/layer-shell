use crate::{Event, event_queue::EventQueue, modules::tray::app::TrayEvent, utils::StringRef};
use app::App;
pub use icon::{TrayIcon, TrayIconPixmap};
pub use item::TrayItem;
use mini_sansio_dbus::IncomingMessage;
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
}

impl Tray {
    pub(crate) fn new() -> Self {
        Self {
            status_notifier_watcher: StatusNotifierWatcher::new(),
            name_lost_or_changed: NameLostOrNameOwnerChanged::new(),
            registry: HashMap::new(),
        }
    }

    pub(crate) fn init(&mut self) {
        self.status_notifier_watcher.init();
        self.name_lost_or_changed.init();
    }

    pub(crate) fn on_message(&mut self, message: IncomingMessage<'_>) {
        if let Some(service) = self.status_notifier_watcher.on_message(message) {
            log::info!(target: "Tray", "Added {service:?}");
            let mut tray_app = App::new(service.clone());
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
            EventQueue::push_back(Event::TrayAppRemoved {
                service: StringRef::new(service.as_str()),
            })
        }

        for (service, app) in &mut self.registry {
            if let Some(event) = app.on_message(message) {
                let service = service.name();

                let event = match event {
                    TrayEvent::Initialized(icon, layout) => Event::TrayAppAdded {
                        service: StringRef::new(service.as_str()),
                        items: layout.into(),
                        icon,
                    },
                    TrayEvent::IconUpdated(icon) => Event::TrayAppIconUpdated {
                        service: StringRef::new(service.as_str()),
                        icon,
                    },
                    TrayEvent::MenuUpdated(layout) => Event::TrayAppMenuUpdated {
                        service: StringRef::new(service.as_str()),
                        items: layout.into(),
                    },
                };
                EventQueue::push_back(event);
            }
        }
    }

    pub(crate) fn trigger(&self, uuid: StringRef) {
        let Ok((service, id)) = UUID::decode(uuid.clone()) else {
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
