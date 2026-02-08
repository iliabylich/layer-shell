use crate::{
    Event,
    dbus::{DBus, Message},
    modules::tray::app::TrayEvent,
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
}

impl Tray {
    pub(crate) fn new() -> Box<Self> {
        Box::new(Self {
            status_notifier_watcher: StatusNotifierWatcher::new(),
            name_lost_or_changed: NameLostOrNameOwnerChanged::new(),
            registry: HashMap::new(),
        })
    }

    pub(crate) fn init(&mut self, dbus: &mut DBus) {
        self.status_notifier_watcher.init(dbus);
        self.name_lost_or_changed.init(dbus);
    }

    pub(crate) fn on_message(
        &mut self,
        dbus: &mut DBus,
        message: &Message,
        events: &mut Vec<Event>,
    ) {
        if let Some(service) = self.status_notifier_watcher.on_message(dbus, message) {
            log::info!("Added {service:?}");
            let mut tray_app = App::new(service.clone());
            tray_app.init(dbus);
            self.registry.insert(service, tray_app);
            return;
        }

        if let Some(address) = self.name_lost_or_changed.on_message(message) {
            let Some(key) = self
                .registry
                .keys()
                .find(|k| k.name() == address || k.raw_address() == address)
                .cloned()
            else {
                return;
            };

            let Some(mut tray_app) = self.registry.remove(&key) else {
                return;
            };

            log::info!("Removed {address}");
            tray_app.reset(dbus);
            events.push(Event::TrayAppRemoved {
                service: address.into(),
            })
        }

        for (service, app) in &mut self.registry {
            if let Some(event) = app.on_message(message, dbus) {
                let service = service.name().to_string().into();

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
                events.push(event);
            }
        }
    }

    pub(crate) fn trigger(&self, uuid: &str, dbus: &mut DBus) {
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
            log::info!("service {service} doesn't exist");
            return;
        };

        let Some(tray_app) = self.registry.get(&key) else {
            log::info!("service {service} doesn't exist");
            return;
        };

        tray_app.trigger(id, dbus);
    }
}
