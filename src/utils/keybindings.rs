use std::collections::HashMap;

use gtk4::{
    prelude::{EventControllerExt, WidgetExt},
    EventControllerKey, Window,
};

pub(crate) struct Keybindings {
    window: &'static Window,
    map: HashMap<&'static str, Box<dyn Fn()>>,
    fallback: Option<Box<dyn Fn(&str)>>,
}

impl Keybindings {
    pub(crate) fn add<F>(mut self, key: &'static str, f: F) -> Self
    where
        F: Fn() + 'static,
    {
        self.map.insert(key, Box::new(f));
        self
    }

    pub(crate) fn fallback<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) + 'static,
    {
        self.fallback = Some(Box::new(f));
        self
    }

    pub(crate) fn finish(self) {
        let Self {
            window,
            map,
            fallback,
        } = self;
        let ctrl = EventControllerKey::new();
        ctrl.connect_key_pressed(move |_, keyval, _, _| {
            let key = keyval.name().unwrap().to_string();
            match map.get(key.as_str()) {
                Some(f) => f(),
                None => {
                    if let Some(fallback) = fallback.as_ref() {
                        fallback(key.as_str())
                    }
                }
            }
            gtk4::glib::Propagation::Proceed
        });
        ctrl.set_propagation_phase(gtk4::PropagationPhase::Capture);
        window.add_controller(ctrl);
    }
}

pub(crate) fn keybindings(window: &'static Window) -> Keybindings {
    Keybindings {
        window,
        map: HashMap::new(),
        fallback: None,
    }
}
