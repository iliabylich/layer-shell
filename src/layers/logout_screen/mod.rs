use crate::{
    globals::load_widget,
    utils::{keybindings, layer_window, singleton, LayerOptions, ToggleWindow},
};
use gtk4::{
    prelude::{GtkWindowExt, WidgetExt},
    Application, Window,
};

mod buttons;

pub(crate) struct LogoutScreen {
    reset: Box<dyn Fn()>,
}
singleton!(LogoutScreen);

impl LogoutScreen {
    const NAME: &str = "LogoutScreen";

    pub(crate) fn activate(app: &Application) {
        let window = load_widget::<Window>(Self::NAME);
        window.set_application(Some(app));
        layer_window(
            window,
            LayerOptions::builder()
                .with_namespace(Self::NAME)
                .with_layer(gtk4_layer_shell::Layer::Overlay)
                .with_anchors(&[
                    gtk4_layer_shell::Edge::Top,
                    gtk4_layer_shell::Edge::Right,
                    gtk4_layer_shell::Edge::Bottom,
                    gtk4_layer_shell::Edge::Left,
                ])
                .with_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive)
                .build(),
        );

        let (reset, on_key_press) = buttons::init();

        keybindings(window)
            .add("Escape", || Self::toggle())
            .fallback(on_key_press)
            .finish();

        window.present();
        window.set_visible(false);

        Self::set(Self { reset })
    }
}

impl ToggleWindow for LogoutScreen {
    fn reset(&self) {
        (self.reset)()
    }

    fn window(&self) -> &'static Window {
        load_widget::<Window>(Self::NAME)
    }
}
