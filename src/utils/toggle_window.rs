use crate::utils::Singleton;
use gtk4::{prelude::WidgetExt, Window};

pub(crate) trait ToggleWindow: Singleton + 'static {
    fn toggle() {
        let instance = Self::get();
        let window = instance.window();

        if !window.get_visible() {
            instance.reset();
        }
        window.set_visible(!window.get_visible())
    }
    fn reset(&self);
    fn window(&self) -> &'static Window;
}
