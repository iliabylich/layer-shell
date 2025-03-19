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

    pub(crate) fn app_removed(&mut self, id: String) -> Event {
        self.map.retain(|k, _| k.id != id);

        self.as_event()
    }

    fn as_event(&self) -> Event {
        Event::Tray {
            list: self.map.values().cloned().collect::<Vec<_>>().into(),
        }
    }

    pub(crate) fn find(&self, id: &str) -> Option<Item> {
        self.map.keys().find(|k| k.id == id).cloned()
    }
}
