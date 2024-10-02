use crate::utils::{keybindings, LayerWindow};
use gtk4::{prelude::WidgetExt, Application};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer};

mod network_list;

pub(crate) struct Networks;

impl LayerWindow for Networks {
    const NAME: &'static str = "Networks";
    const LAYER: Layer = Layer::Overlay;
    const ANCHORS: &'static [Edge] = &[Edge::Top, Edge::Right];
    const MARGINS: &'static [(Edge, i32)] = &[(Edge::Top, 50)];
    const KEYBOARD_MODE: Option<KeyboardMode> = Some(KeyboardMode::Exclusive);

    fn reset() {}
}

impl Networks {
    pub(crate) fn activate(app: &Application) {
        let window = Self::layer_window(app);

        network_list::init();

        keybindings(window)
            .add("Escape", || window.set_visible(false))
            .fallback(|_| {})
            .finish();
    }
}
