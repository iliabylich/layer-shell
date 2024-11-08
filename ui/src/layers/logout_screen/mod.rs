use crate::{utils::keybindings, widgets::LogoutScreenWindow};
use gtk4::{
    prelude::{GtkWindowExt, WidgetExt},
    Application,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use layer_shell_io::{subscribe, Event};
use layer_shell_utils::global;

mod buttons;

pub(crate) struct LogoutScreen;
global!(RESET, Box<dyn Fn()>);

impl LogoutScreen {
    pub(crate) fn activate(app: &Application) {
        let window = LogoutScreenWindow();

        window.set_application(Some(app));

        LayerShell::init_layer_shell(window);
        LayerShell::set_layer(window, Layer::Overlay);
        LayerShell::set_anchor(window, Edge::Top, true);
        LayerShell::set_anchor(window, Edge::Right, true);
        LayerShell::set_anchor(window, Edge::Bottom, true);
        LayerShell::set_anchor(window, Edge::Left, true);
        LayerShell::set_namespace(window, "LogoutScreen");
        LayerShell::set_keyboard_mode(window, KeyboardMode::Exclusive);

        let (reset, on_key_press) = buttons::init();

        keybindings(window)
            .add("Escape", Self::toggle)
            .fallback(on_key_press)
            .finish();

        window.present();
        window.set_visible(false);

        subscribe(on_event);

        RESET::set(reset)
    }

    pub(crate) fn toggle() {
        let window = LogoutScreenWindow();

        if !window.get_visible() {
            (RESET::get())();
        }
        window.set_visible(!window.get_visible())
    }
}

fn on_event(event: &Event) {
    if let Event::ToggleLogoutScreen = event {
        LogoutScreen::toggle();
    }
}
