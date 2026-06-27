use crate::{
    Event,
    event_queue::EventQueue,
    modules::tray::app::TrayEvent,
    utils::{StringRef, StringRefExt, dbus::queue::SessionDBusQueue},
};
use anyhow::Result;
use app::App;
use dbus::{
    IncomingMessage,
    messages::org_freedesktop_dbus::{NameOwnerChangedSignal, NameOwnerChangedSubscribe},
    messaging::DBusEncode as _,
};
pub use icon::{TrayIcon, TrayIconPixmap};
pub use item::TrayItem;
use service::Service;
use status_notifier_watcher::StatusNotifierWatcher;
use std::collections::HashMap;
use uuid::UUID;

mod app;
mod icon;
mod item;
mod service;
mod status_notifier_watcher;
mod uuid;

pub(crate) struct Tray {
    registry: HashMap<Service, App>,
}

impl Tray {
    pub(crate) fn new(q: &mut SessionDBusQueue) -> Result<Self> {
        Self::init(q)?;
        Ok(Self {
            registry: HashMap::new(),
        })
    }

    fn init(q: &mut SessionDBusQueue) -> Result<()> {
        StatusNotifierWatcher::request_ksni_name(q)?;

        let mut buf = [0; 1_024];
        let buf = NameOwnerChangedSubscribe::encode((), &mut buf)?;
        q.push_raw(buf);
        Ok(())
    }

    pub(crate) fn handle(
        &mut self,
        message: IncomingMessage<'_>,
        events: &mut EventQueue,
        q: &mut SessionDBusQueue,
    ) {
        if let Err(err) = self.try_handle(message, events, q) {
            log::error!("{err:?}");
        }
    }

    fn try_handle(
        &mut self,
        message: IncomingMessage<'_>,
        events: &mut EventQueue,
        q: &mut SessionDBusQueue,
    ) -> Result<()> {
        if let Some(service) = StatusNotifierWatcher::handle_incoming_request(message, q)? {
            log::info!(target: "Tray", "Added {service:?}");
            let mut tray_app = App::new(service.clone());
            tray_app.init(q)?;
            self.registry.insert(service, tray_app);
            return Ok(());
        }

        if let Some(service) = NameOwnerChangedSignal::handle(message)? {
            let Some(key) = self
                .registry
                .keys()
                .find(|s| s.name() == service || s.raw_address() == service)
                .cloned()
            else {
                return Ok(());
            };

            let Some(mut tray_app) = self.registry.remove(&key) else {
                return Ok(());
            };

            log::info!(target: "Tray", "Removed {service}");
            tray_app.reset(q)?;
            events.push_back(Event::TrayAppRemoved {
                service: StringRef::new(service),
            });
        }

        for (service, app) in &mut self.registry {
            if let Some(event) = app.handle(message, q)? {
                let service = service.name();

                let event = match event {
                    TrayEvent::Initialized(icon, layout) => Event::TrayAppAdded {
                        service: service.clone(),
                        items: layout.into(),
                        icon,
                    },
                    TrayEvent::IconUpdated(icon) => Event::TrayAppIconUpdated {
                        service: service.clone(),
                        icon,
                    },
                    TrayEvent::MenuUpdated(layout) => Event::TrayAppMenuUpdated {
                        service: service.clone(),
                        items: layout.into(),
                    },
                };
                events.push_back(event);
            }
        }

        Ok(())
    }

    fn try_trigger(&self, uuid: &str, q: &mut SessionDBusQueue) -> Result<()> {
        let Ok((service, id)) = UUID::decode(uuid) else {
            log::error!("malformed UUID: {uuid:?}");
            return Ok(());
        };

        let Some(key) = self
            .registry
            .keys()
            .find(|k| k.name() == service || k.raw_address() == service)
            .cloned()
        else {
            log::info!(target: "Tray", "service {service} doesn't exist");
            return Ok(());
        };

        let Some(tray_app) = self.registry.get(&key) else {
            log::info!(target: "Tray", "service {service} doesn't exist");
            return Ok(());
        };

        tray_app.trigger(id, q)?;
        Ok(())
    }

    pub(crate) fn trigger(&self, uuid: &str, q: &mut SessionDBusQueue) {
        if let Err(err) = self.try_trigger(uuid, q) {
            log::error!("{err:?}");
        }
    }
}
