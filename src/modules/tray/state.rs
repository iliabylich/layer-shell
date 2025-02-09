use crate::{event::TrayApp, modules::tray::item::Item, Event};
use std::{collections::HashMap, sync::mpsc::Sender};

#[derive(Debug)]
pub(crate) struct State {
    map: HashMap<Item, TrayApp>,
    tx: Sender<Event>,
}

impl State {
    pub(crate) fn new(tx: Sender<Event>) -> Self {
        Self {
            map: HashMap::new(),
            tx,
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

    fn emit(&self) {
        let event = Event::Tray {
            list: self.map.values().cloned().collect::<Vec<_>>().into(),
        };
        if let Err(err) = self.tx.send(event) {
            log::error!("failed to send Tray event: {:?}", err);
        }
    }

    pub(crate) fn find(&self, service: &str) -> Option<Item> {
        self.map.keys().find(|k| k.service == service).cloned()
    }
}
