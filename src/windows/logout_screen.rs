use gtk4::{prelude::GtkWindowExt, Application, Window};

use crate::{
    utils::{layer_window, load_widget, LayerOptions},
    windows::GloballyAccessibleWindow,
};

pub(crate) struct LogoutScreen;

impl GloballyAccessibleWindow for LogoutScreen {
    const NAME: &str = "LogoutScreen";
}

impl LogoutScreen {
    pub(crate) fn activate(app: &Application) {
        let window: Window = load_widget("LogoutScreen");
        window.set_application(Some(app));
        layer_window(
            &window,
            LayerOptions::builder()
                .with_namespace("LogoutScreen")
                .with_layer(gtk4_layer_shell::Layer::Overlay)
                .with_anchors(&[
                    gtk4_layer_shell::Edge::Top,
                    gtk4_layer_shell::Edge::Right,
                    gtk4_layer_shell::Edge::Bottom,
                    gtk4_layer_shell::Edge::Left,
                ])
                // .with_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive)
                .build(),
        );

        Self::set(window);
    }
}