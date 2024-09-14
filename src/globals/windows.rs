use crate::{globals::load_widget, models::singleton};
use gtk4::prelude::WidgetExt;
use std::collections::HashMap;

pub(crate) struct GlobalWindows {
    map: HashMap<&'static str, Box<dyn Fn()>>,
}
singleton!(GlobalWindows);

impl GlobalWindows {
    pub(crate) fn init() {
        Self::set(Self {
            map: HashMap::new(),
        })
    }

    pub(crate) fn set_reset_fn<F>(name: &'static str, f: F)
    where
        F: Fn() + 'static,
    {
        Self::get().map.insert(name, Box::new(f));
    }

    fn get_reset_fn(name: &'static str) -> &'static dyn Fn() {
        Self::get().map.get(name).unwrap()
    }
}

pub(crate) fn toggle_window(name: &'static str) {
    let window = load_widget::<gtk4::Window>(name);
    let reset_fn = GlobalWindows::get_reset_fn(name);
    if !window.get_visible() {
        reset_fn();
    }
    window.set_visible(!window.get_visible())
}
