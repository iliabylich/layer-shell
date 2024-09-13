use gtk4::{prelude::GtkWindowExt, Application, Window};

use crate::{
    utils::{layer_window, load_widget, LayerOptions},
    windows::GloballyAccessibleWindow,
};

pub(crate) struct Networks;

impl GloballyAccessibleWindow for Networks {
    const NAME: &str = "Networks";
}

impl Networks {
    pub(crate) fn activate(app: &Application) {
        let window: Window = load_widget("Networks");
        window.set_application(Some(app));
        layer_window(
            &window,
            LayerOptions::builder()
                .with_namespace("Networks")
                .with_layer(gtk4_layer_shell::Layer::Overlay)
                .with_anchors(&[gtk4_layer_shell::Edge::Top, gtk4_layer_shell::Edge::Right])
                .with_margins(&[(gtk4_layer_shell::Edge::Top, 50)])
                // .with_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive)
                .build(),
        );

        Self::set(window);
    }
}