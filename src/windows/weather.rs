use gtk4::{
    prelude::{GtkWindowExt, WidgetExt},
    Application, Window,
};

use crate::{
    globals::{load_widget, GlobalWindows},
    utils::{keybindings, layer_window, LayerOptions},
    widgets::WeatherForecast,
};

pub(crate) struct Weather;

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

        let widget = WeatherForecast::init();

        keybindings(window)
            .add("Escape", || window.set_visible(false))
            .fallback(|_key| {})
            .finish();

        GlobalWindows::set_reset_fn("Weather", move || {
            (widget.reset)();
        });
    }
}
