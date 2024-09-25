use crate::{
    globals::load_widget,
    utils::{keybindings, layer_window, singleton, LayerOptions, ToggleWindow},
};
use gtk4::{
    prelude::{GtkWindowExt, WidgetExt},
    Application, Window,
};

mod app_list;

pub(crate) struct Launcher {
    reset: Box<dyn Fn()>,
}
singleton!(Launcher);

impl Launcher {
    const NAME: &str = "Launcher";

    pub(crate) fn activate(app: &Application) {
        let window: &Window = load_widget(Self::NAME);
        window.set_application(Some(app));
        layer_window(
            window,
            LayerOptions::builder()
                .with_namespace(Self::NAME)
                .with_layer(gtk4_layer_shell::Layer::Overlay)
                .with_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive)
                .build(),
        );

        let (reset, on_key_press) = app_list::init();

        keybindings(window)
            .add("Escape", || Self::toggle())
            .fallback(on_key_press)
            .finish();

        window.present();
        window.set_visible(false);

        Self::set(Self { reset });
    }
}

impl ToggleWindow for Launcher {
    fn reset(&self) {
        (self.reset)();
    }

    fn window(&self) -> &'static Window {
        load_widget(Self::NAME)
    }
}
