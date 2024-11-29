use crate::{utils::keybindings, widgets::session::Window};
use gtk4::{
    prelude::{GtkWindowExt, WidgetExt},
    Application,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use layer_shell_io::{subscribe, Event};

mod buttons;

pub(crate) struct SessionScreen;

impl SessionScreen {
    pub(crate) fn activate(app: &Application) {
        let window = Window();

        window.set_application(Some(app));

        LayerShell::init_layer_shell(window);
        LayerShell::set_layer(window, Layer::Overlay);
        LayerShell::set_anchor(window, Edge::Top, true);
        LayerShell::set_anchor(window, Edge::Right, true);
        LayerShell::set_anchor(window, Edge::Bottom, true);
        LayerShell::set_anchor(window, Edge::Left, true);
        LayerShell::set_namespace(window, "LayerShell/SessionScreen");
        LayerShell::set_keyboard_mode(window, KeyboardMode::Exclusive);

        buttons::init();

        keybindings(window)
            .add("Escape", || window.set_visible(false))
            .finish();

        window.present();
        window.set_visible(false);

        subscribe(|event| {
            if let Event::ToggleSessionScreen = event {
                Self::toggle();
            }
        });
    }

    pub(crate) fn toggle() {
        Window().set_visible(!Window().get_visible())
    }
}
