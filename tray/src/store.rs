use crate::{TrayAppRemovedEvent, TrayAppUpdatedEvent, TrayEvent, TrayIcon, TrayItem};
use std::{collections::HashMap, sync::Arc};

#[derive(Default)]
pub(crate) struct Store {
    icons: HashMap<Arc<str>, TrayIcon>,
    items: HashMap<Arc<str>, TrayItem>,
}

impl Store {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn update_icon(&mut self, service: Arc<str>, icon: TrayIcon) -> Option<TrayEvent> {
        self.icons.insert(Arc::clone(&service), icon);
        self.event_if_service_has_complete_def(service)
    }

    pub(crate) fn update_item(&mut self, service: Arc<str>, item: TrayItem) -> Option<TrayEvent> {
        self.items.insert(Arc::clone(&service), item);
        self.event_if_service_has_complete_def(service)
    }

    fn event_if_service_has_complete_def(&self, service: Arc<str>) -> Option<TrayEvent> {
        let icon = self.icons.get(&service)?.clone();
        let item = self.items.get(&service)?.clone();

        Some(TrayEvent::AppUpdated(TrayAppUpdatedEvent {
            service: service.to_string().into(),
            root_item: item,
            icon,
        }))
    }

    pub(crate) fn remove(&mut self, service: Arc<str>) -> Option<TrayEvent> {
        let mut emit = false;
        emit |= self.icons.remove(&service).is_some();
        emit |= self.items.remove(&service).is_some();

        if emit {
            Some(TrayEvent::AppRemoved(TrayAppRemovedEvent {
                service: service.to_string().into(),
            }))
        } else {
            None
        }
    }
}
