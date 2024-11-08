use crate::{utils::keybindings, widgets::LauncherWindow};
use gtk4::{
    prelude::{GtkWindowExt, WidgetExt},
    Application,
};
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};
use layer_shell_io::{subscribe, Event};
use layer_shell_utils::global;

mod app_list;

pub(crate) struct Launcher;
global!(RESET, Box<dyn Fn()>);

impl Launcher {
    pub(crate) fn activate(app: &Application) {
        let window = LauncherWindow();

        window.set_application(Some(app));

        LayerShell::init_layer_shell(window);
        LayerShell::set_layer(window, Layer::Overlay);
        LayerShell::set_namespace(window, "Launcher");
        LayerShell::set_keyboard_mode(window, KeyboardMode::Exclusive);

        let (reset, on_key_press) = app_list::init();

        keybindings(window)
            .add("Escape", Self::toggle)
            .fallback(on_key_press)
            .finish();

        window.present();
        window.set_visible(false);

        subscribe(on_event);

        RESET::set(reset);
    }

    pub(crate) fn toggle() {
        let window = LauncherWindow();

        if !window.get_visible() {
            (RESET::get())();
        }
        window.set_visible(!window.get_visible())
    }
}

fn on_event(event: &Event) {
    if let Event::ToggleLauncher = event {
        Launcher::toggle();
    }
}
