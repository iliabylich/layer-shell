use crate::globals::load_widget;
use gtk4::prelude::WidgetExt;
use std::collections::HashMap;

static mut WINDOW_TO_RESET_FN: Option<HashMap<&'static str, Box<dyn Fn()>>> = None;

pub(crate) struct GlobalWindows;

impl GlobalWindows {
    pub(crate) fn set_reset_fn<F>(name: &'static str, f: F)
    where
        F: Fn() + 'static,
    {
        unsafe {
            if WINDOW_TO_RESET_FN.is_none() {
                WINDOW_TO_RESET_FN = Some(HashMap::new());
            }

            let map = WINDOW_TO_RESET_FN.as_mut().unwrap();

            map.insert(name, Box::new(f));
        }
    }

    fn get_reset_fn(name: &'static str) -> &'static dyn Fn() {
        unsafe { WINDOW_TO_RESET_FN.as_ref().unwrap().get(name).unwrap() }
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
