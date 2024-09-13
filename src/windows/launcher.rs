use gtk4::{prelude::GtkWindowExt, Application, Window};

use crate::{
    globals::load_widget,
    utils::{layer_window, LayerOptions},
    windows::GloballyAccessibleWindow,
};

pub(crate) struct Launcher;

impl GloballyAccessibleWindow for Launcher {
    const NAME: &str = "Launcher";
}

impl Launcher {
    pub(crate) fn activate(app: &Application) {
        let window: &Window = load_widget("Launcher");
        window.set_application(Some(app));
        layer_window(
            &window,
            LayerOptions::builder()
                .with_namespace("Launcher")
                .with_layer(gtk4_layer_shell::Layer::Overlay)
                // .with_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive)
                .build(),
        );

        Self::set(window);
    }
}
