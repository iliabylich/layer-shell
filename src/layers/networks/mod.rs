use crate::{
    globals::load_widget,
    utils::{keybindings, layer_window, singleton, LayerOptions, ToggleWindow},
};
use gtk4::{
    prelude::{GtkWindowExt, WidgetExt},
    Application, Window,
};

mod network_list;

pub(crate) struct Networks {
    reset: Box<dyn Fn()>,
}
singleton!(Networks);

impl Networks {
    pub(crate) fn activate(app: &Application) {
        let window = load_widget::<Window>("Networks");
        window.set_application(Some(app));
        layer_window(
            window,
            LayerOptions::builder()
                .with_namespace("Networks")
                .with_layer(gtk4_layer_shell::Layer::Overlay)
                .with_anchors(&[gtk4_layer_shell::Edge::Top, gtk4_layer_shell::Edge::Right])
                .with_margins(&[(gtk4_layer_shell::Edge::Top, 50)])
                .with_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive)
                .build(),
        );

        let (reset, on_key_press) = network_list::init();

        keybindings(window)
            .add("Escape", || window.set_visible(false))
            .fallback(on_key_press)
            .finish();

        Self::set(Self { reset })
    }
}

impl ToggleWindow for Networks {
    fn reset(&self) {
        (self.reset)()
    }

    fn window(&self) -> &'static Window {
        load_widget("Networks")
    }
}
