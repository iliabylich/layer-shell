use crate::utils::{keybindings, LayerWindow};
use gtk4::{prelude::WidgetExt, Application};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer};

mod forecast;

pub(crate) struct Weather;

impl LayerWindow for Weather {
    const NAME: &'static str = "Weather";
    const LAYER: Layer = Layer::Overlay;
    const ANCHORS: &'static [Edge] = &[Edge::Top, Edge::Right];
    const MARGINS: &'static [(Edge, i32)] = &[(Edge::Top, 50), (Edge::Right, 800)];
    const KEYBOARD_MODE: Option<KeyboardMode> = Some(KeyboardMode::Exclusive);

    fn reset() {}
}

impl Weather {
    pub(crate) fn activate(app: &Application) {
        let window = Self::layer_window(app);

        forecast::init();

        keybindings(window)
            .add("Escape", || window.set_visible(false))
            .fallback(|_| {})
            .finish();
    }
}
