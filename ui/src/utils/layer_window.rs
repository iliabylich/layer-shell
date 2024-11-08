use gtk4::{
    prelude::{GtkWindowExt, WidgetExt},
    Application, Window,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

pub(crate) trait LayerWindow {
    const NAME: &'static str;
    const LAYER: Layer;
    const ANCHORS: &'static [Edge];
    const MARGINS: &'static [(Edge, i32)];
    const KEYBOARD_MODE: Option<KeyboardMode>;

    fn layer_window(app: &Application) -> &'static Window {
        let w = Self::window();
        w.set_application(Some(app));

        LayerShell::init_layer_shell(w);
        LayerShell::set_layer(w, Self::LAYER);
        for edge in Self::ANCHORS {
            LayerShell::set_anchor(w, *edge, true);
        }
        for (edge, margin) in Self::MARGINS {
            LayerShell::set_margin(w, *edge, *margin);
        }
        LayerShell::set_namespace(w, Self::NAME);
        if let Some(keyboard_mode) = Self::KEYBOARD_MODE {
            LayerShell::set_keyboard_mode(w, keyboard_mode)
        }

        w
    }

    fn window() -> &'static gtk4::Window;

    fn toggle() {
        let window = Self::window();

        if !window.get_visible() {
            Self::reset();
        }
        window.set_visible(!window.get_visible())
    }
    fn reset();
}
