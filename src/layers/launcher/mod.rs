use crate::utils::{keybindings, singleton, LayerWindow};
use gtk4::{
    prelude::{GtkWindowExt, WidgetExt},
    Application,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer};

mod app_list;

pub(crate) struct Launcher {
    reset: Box<dyn Fn()>,
}
singleton!(Launcher);

impl LayerWindow for Launcher {
    const NAME: &str = "Launcher";
    const LAYER: Layer = Layer::Overlay;
    const ANCHORS: &[Edge] = &[];
    const MARGINS: &[(Edge, i32)] = &[];
    const KEYBOARD_MODE: Option<KeyboardMode> = Some(KeyboardMode::Exclusive);

    fn reset(&self) {
        (self.reset)();
    }
}

impl Launcher {
    pub(crate) fn activate(app: &Application) {
        let window = Self::layer_window(app);

        let (reset, on_key_press) = app_list::init();

        keybindings(window)
            .add("Escape", Self::toggle)
            .fallback(on_key_press)
            .finish();

        window.present();
        window.set_visible(false);

        Self::set(Self { reset });
    }
}
