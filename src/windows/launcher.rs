use gtk4::{
    prelude::{GtkWindowExt, WidgetExt},
    Application, Window,
};

use crate::{
    globals::{load_widget, GlobalWindows},
    utils::{keybindings, layer_window, LayerOptions},
    widgets::AppList,
};

pub(crate) struct Launcher;

impl Launcher {
    pub(crate) fn activate(app: &Application) {
        let window: &Window = load_widget("Launcher");
        window.set_application(Some(app));
        layer_window(
            window,
            LayerOptions::builder()
                .with_namespace("Launcher")
                .with_layer(gtk4_layer_shell::Layer::Overlay)
                .with_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive)
                .build(),
        );

        let widget = AppList::init();

        keybindings(window)
            .add("Escape", || window.set_visible(false))
            .fallback(move |key| (widget.on_key_press)(key))
            .finish();

        GlobalWindows::set_reset_fn("Launcher", move || {
            (widget.reset)();
        });
    }
}
