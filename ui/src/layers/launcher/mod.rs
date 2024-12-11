use crate::{utils::keybindings, widgets::launcher::Window};
use gtk4::{
    prelude::{GtkWindowExt, WidgetExt},
    Application,
};
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};
use layer_shell_app_list::{AppListGoDown, AppListGoUp, AppListReset};
use layer_shell_io::{publish, subscribe, Command, Event};

mod app_list;

pub(crate) struct Launcher;

impl Launcher {
    pub(crate) fn activate(app: &Application) {
        let window = Window();

        window.set_application(Some(app));

        LayerShell::init_layer_shell(window);
        LayerShell::set_layer(window, Layer::Overlay);
        LayerShell::set_namespace(window, "LayerShell/Launcher");
        LayerShell::set_keyboard_mode(window, KeyboardMode::Exclusive);

        app_list::init();

        keybindings(window)
            .add("Escape", || window.set_visible(false))
            .add("Up", || publish(Command::AppListGoUp(AppListGoUp)))
            .add("Down", || publish(Command::AppListGoDown(AppListGoDown)))
            .finish();

        window.present();
        window.set_visible(false);

        subscribe(|event| {
            if let Event::ToggleLauncher = event {
                Self::toggle();
            }
        });
    }

    pub(crate) fn toggle() {
        let window = Window();

        if !window.get_visible() {
            publish(Command::AppListReset(AppListReset));
            app_list::reset();
        }
        window.set_visible(!window.get_visible())
    }
}
