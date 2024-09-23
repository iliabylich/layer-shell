use crate::{
    globals::load_widget,
    utils::{keybindings, layer_window, singleton, LayerOptions, ToggleWindow},
};
use gtk4::{
    prelude::{GtkWindowExt, WidgetExt},
    Application, Window,
};

mod forecast;

pub(crate) struct Weather {
    reset: Box<dyn Fn()>,
}
singleton!(Weather);

impl Weather {
    pub(crate) fn activate(app: &Application) {
        let window = load_widget::<Window>("Weather");
        window.set_application(Some(app));

        layer_window(
            window,
            LayerOptions::builder()
                .with_namespace("Weather")
                .with_layer(gtk4_layer_shell::Layer::Overlay)
                .with_anchors(&[gtk4_layer_shell::Edge::Top, gtk4_layer_shell::Edge::Right])
                .with_margins(&[
                    (gtk4_layer_shell::Edge::Top, 50),
                    (gtk4_layer_shell::Edge::Right, 800),
                ])
                .with_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive)
                .build(),
        );

        let (reset, on_key_press) = forecast::init();

        keybindings(window)
            .add("Escape", || window.set_visible(false))
            .fallback(on_key_press)
            .finish();

        Self::set(Self { reset })
    }
}

impl ToggleWindow for Weather {
    fn reset(&self) {
        (self.reset)();
    }

    fn window(&self) -> &'static Window {
        load_widget::<Window>("Weather")
    }
}
