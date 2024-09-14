use gtk4::{
    prelude::{GtkWindowExt, WidgetExt},
    Application, Window,
};

use crate::{
    globals::{load_widget, GlobalWindows},
    utils::{keybindings, layer_window, LayerOptions},
    widgets::Logout,
};

pub(crate) struct LogoutScreen;

impl LogoutScreen {
    pub(crate) fn activate(app: &Application) {
        let window: &Window = load_widget("LogoutScreen");
        window.set_application(Some(app));
        layer_window(
            window,
            LayerOptions::builder()
                .with_namespace("LogoutScreen")
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

        let widget = Logout::init();

        keybindings(window)
            .add("Escape", || window.set_visible(false))
            .fallback(move |key| (widget.on_key_press)(key))
            .finish();

        GlobalWindows::set_reset_fn("LogoutScreen", move || {
            (widget.reset)();
        });
    }
}
