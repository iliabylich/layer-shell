use crate::utils::{global, keybindings, LayerWindow};
use gtk4::{prelude::WidgetExt, Application};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer};

mod network_list;

pub(crate) struct Networks;
global!(RESET, Box<dyn Fn()>);

impl LayerWindow for Networks {
    const NAME: &'static str = "Networks";
    const LAYER: Layer = Layer::Overlay;
    const ANCHORS: &'static [Edge] = &[Edge::Top, Edge::Right];
    const MARGINS: &'static [(Edge, i32)] = &[(Edge::Top, 50)];
    const KEYBOARD_MODE: Option<KeyboardMode> = Some(KeyboardMode::Exclusive);

    fn reset() {
        (RESET::get())()
    }
}

impl Networks {
    pub(crate) fn activate(app: &Application) {
        let window = Self::layer_window(app);

        let (reset, on_key_press) = network_list::init();

        keybindings(window)
            .add("Escape", || window.set_visible(false))
            .fallback(on_key_press)
            .finish();

        RESET::set(reset)
    }
}
