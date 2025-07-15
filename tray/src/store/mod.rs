mod data;
mod diff;

use crate::{TrayAppRemovedEvent, TrayEvent, TrayIcon, TrayItem, store::data::Data};
use std::{collections::HashMap, sync::Arc};

#[derive(Default)]
pub(crate) struct Store {
    map: HashMap<Arc<str>, Data>,
}

impl Store {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn update_icon(&mut self, service: Arc<str>, icon: TrayIcon) -> Option<TrayEvent> {
        self.map
            .entry(Arc::clone(&service))
            .or_default()
            .set_icon(icon)
            .into_event(service)
    }

    pub(crate) fn update_item(
        &mut self,
        service: Arc<str>,
        items: Vec<TrayItem>,
    ) -> Option<TrayEvent> {
        self.map
            .entry(Arc::clone(&service))
            .or_default()
            .set_items(items)
            .into_event(service)
    }

    pub(crate) fn remove(&mut self, service: Arc<str>) -> Option<TrayEvent> {
        if self.map.remove(&service)?.is_full() {
            Some(TrayEvent::AppRemoved(TrayAppRemovedEvent {
                service: service.into(),
            }))
        } else {
            None
        }
    }
}
