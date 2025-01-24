use crate::{event::TrayApp, modules::tray::item::Item, Event};
use std::collections::HashMap;

pub(crate) struct State {
    map: HashMap<Item, TrayApp>,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub(crate) fn app_added(&mut self, item: Item, app: TrayApp) {
        let changed = if let Some(existing_app) = self.map.get_mut(&item) {
            if *existing_app != app {
                *existing_app = app;
                true
            } else {
                false
            }
        } else {
            self.map.insert(item, app);
            true
        };

        if changed {
            self.emit();
        }
    }

    pub(crate) fn app_removed(&mut self, service: String) {
        let changed = {
            let initial_len = self.map.len();
            self.map.retain(|k, _| k.service != service);
            self.map.len() != initial_len
        };

        if changed {
            self.emit();
        }
    }

    pub(crate) fn emit(&self) {
        let event = Event::Tray {
            list: self.map.values().cloned().collect::<Vec<_>>().into(),
        };
        event.emit();
    }

    pub(crate) fn find(&self, service: &str) -> Option<Item> {
        self.map.keys().find(|k| k.service == service).cloned()
    }
}
