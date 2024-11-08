use std::collections::HashMap;

use gtk4::{
    prelude::{EventControllerExt, WidgetExt},
    EventControllerKey, Window,
};

pub(crate) struct Keybindings {
    window: &'static Window,
    map: HashMap<&'static str, Box<dyn Fn()>>,
}

impl Keybindings {
    pub(crate) fn add<F>(mut self, key: &'static str, f: F) -> Self
    where
        F: Fn() + 'static,
    {
        self.map.insert(key, Box::new(f));
        self
    }

    pub(crate) fn finish(self) {
        let Self { window, map } = self;
        let ctrl = EventControllerKey::new();
        ctrl.connect_key_pressed(move |_, keyval, _, _| {
            if let Some(key) = keyval.name() {
                if let Some(f) = map.get(key.as_str()) {
                    f();
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
    }
}
