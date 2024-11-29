use crate::{utils::keybindings, widgets::htop::Window};
use gtk4::{
    gio::Cancellable,
    prelude::{GtkWindowExt, WidgetExt},
    Application,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use vte4::TerminalExtManual;

pub(crate) struct Htop;

impl Htop {
    pub(crate) fn activate(app: &Application) {
        let window = Window();

        window.set_application(Some(app));

        LayerShell::init_layer_shell(window);
        LayerShell::set_layer(window, Layer::Overlay);
        LayerShell::set_anchor(window, Edge::Top, true);
        LayerShell::set_anchor(window, Edge::Right, true);
        LayerShell::set_margin(window, Edge::Top, 50);
        LayerShell::set_margin(window, Edge::Right, 600);
        LayerShell::set_namespace(window, "LayerShell/Htop");
        LayerShell::set_keyboard_mode(window, KeyboardMode::Exclusive);

        let terminal = vte4::Terminal::builder().build();
        let home = std::env::var("HOME").unwrap_or_else(|err| {
            eprintln!("$HOME is not set: {:?}", err);
            std::process::exit(1);
        });
        terminal.spawn_async(
            vte4::PtyFlags::DEFAULT,
            Some(&home),
            &["htop"],
            &[],
            gtk4::glib::SpawnFlags::DEFAULT,
            || {
                // finished
            },
            -1,
            Cancellable::NONE,
            |_child_pid| {
                // started
            },
        );
        window.set_child(Some(&terminal));

        keybindings(window)
            .add("Escape", || window.set_visible(false))
            .finish();
    }

    pub(crate) fn toggle() {
        Window().set_visible(!Window().get_visible())
    }
}
