use crate::{event::TrayApp, modules::tray::item::Item, Event};
use std::{cell::RefCell, collections::HashMap};

thread_local! {
    static MAP: RefCell<HashMap<Item, TrayApp>> = RefCell::new(HashMap::new());
}

pub(crate) struct State;

impl State {
    pub(crate) fn app_added(item: Item, app: TrayApp) {
        let changed = MAP.with(|map| {
            let mut map = map.borrow_mut();

            if let Some(existing_app) = map.get_mut(&item) {
                if *existing_app != app {
                    *existing_app = app;
                    true
                } else {
                    false
                }
            } else {
                map.insert(item, app);
                true
            }
        });

        if changed {
            Self::emit();
        }
    }

    pub(crate) fn app_removed(service: String) {
        let changed = MAP.with(|map| {
            let mut map = map.borrow_mut();

            let initial_len = map.len();
            map.retain(|k, _| k.service != service);
            map.len() != initial_len
        });

        if changed {
            Self::emit();
        }
    }

    pub(crate) fn emit() {
        MAP.with(|map| {
            let map = map.borrow();
            let event = Event::Tray {
                list: map.values().cloned().collect::<Vec<_>>().into(),
            };
            event.emit();
        });
    }

    pub(crate) fn find(service: &str) -> Option<Item> {
        MAP.with(|map| {
            let map = map.borrow();
            map.keys().find(|k| k.service == service).cloned()
        })
    }
}
