use crate::{Event, event::TrayApp, modules::tray::item::Item};
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

    pub(crate) fn app_added(&mut self, item: Item, app: TrayApp) -> Event {
        self.map.insert(item, app);

        self.as_event()
    }

    pub(crate) fn app_removed(&mut self, service: String) -> Event {
        self.map.retain(|k, _| k.service != service);

        self.as_event()
    }

    fn as_event(&self) -> Event {
        Event::Tray {
            list: self.map.values().cloned().collect::<Vec<_>>().into(),
        }
    }

    pub(crate) fn find(&self, service: &str) -> Option<Item> {
        self.map
            .keys()
            .find(|k| k.service == service || k.service_id == service)
            .cloned()
    }
}
