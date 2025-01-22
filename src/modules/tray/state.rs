use crate::{event::TrayApp, Event};
use std::{cell::RefCell, collections::HashMap};

#[derive(Debug, Clone)]
pub(crate) struct State {
    apps: HashMap<String, TrayApp>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State { apps: HashMap::new() });
}

impl State {
    pub(crate) fn app_added(service: impl Into<String>, app: TrayApp) {
        let service = service.into();

        STATE.with(|state| {
            let mut state = state.borrow_mut();
            if let std::collections::hash_map::Entry::Vacant(e) = state.apps.entry(service) {
                e.insert(app);
                state.emit();
            }
        });
    }

    pub(crate) fn app_removed(service: impl AsRef<str>) {
        let service = service.as_ref();

        STATE.with(|state| {
            let mut state = state.borrow_mut();
            if state.apps.contains_key(service) {
                state.apps.remove(service);
                state.emit();
            }
        });
    }

    pub(crate) fn emit(&self) {
        let event = Event::Tray {
            list: self.apps.values().cloned().collect::<Vec<_>>().into(),
        };
        event.emit();
    }
}
