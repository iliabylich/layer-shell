use crate::utils::{keybindings, singleton, LayerWindow};
use gtk4::{
    prelude::{GtkWindowExt, WidgetExt},
    Application,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer};

mod buttons;

pub(crate) struct LogoutScreen {
    reset: Box<dyn Fn()>,
}
singleton!(LogoutScreen);

impl LayerWindow for LogoutScreen {
    const NAME: &str = "LogoutScreen";
    const LAYER: Layer = Layer::Overlay;
    const ANCHORS: &[Edge] = &[Edge::Top, Edge::Right, Edge::Bottom, Edge::Left];
    const MARGINS: &[(Edge, i32)] = &[];
    const KEYBOARD_MODE: Option<KeyboardMode> = Some(KeyboardMode::Exclusive);

    fn reset(&self) {
        (self.reset)()
    }
}

impl LogoutScreen {
    pub(crate) fn activate(app: &Application) {
        let window = Self::layer_window(app);

        let (reset, on_key_press) = buttons::init();

        keybindings(window)
            .add("Escape", Self::toggle)
            .fallback(on_key_press)
            .finish();

        window.present();
        window.set_visible(false);

        Self::set(Self { reset })
    }
}
