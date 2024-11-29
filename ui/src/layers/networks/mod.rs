use crate::{utils::keybindings, widgets::networks::Window};
use gtk4::{
    prelude::{GtkWindowExt, WidgetExt},
    Application,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

mod network_list;

pub(crate) struct Networks;

impl Networks {
    pub(crate) fn activate(app: &Application) {
        let window = Window();

        window.set_application(Some(app));

        LayerShell::init_layer_shell(window);
        LayerShell::set_layer(window, Layer::Overlay);
        LayerShell::set_anchor(window, Edge::Top, true);
        LayerShell::set_anchor(window, Edge::Right, true);
        LayerShell::set_margin(window, Edge::Top, 50);
        LayerShell::set_namespace(window, "LayerShell/Networks");
        LayerShell::set_keyboard_mode(window, KeyboardMode::Exclusive);

        network_list::init();

        keybindings(window)
            .add("Escape", || window.set_visible(false))
            .finish();
    }

    pub(crate) fn toggle() {
        Window().set_visible(!Window().get_visible())
    }
}
