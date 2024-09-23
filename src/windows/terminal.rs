use crate::{
    globals::{load_widget, GlobalWindows},
    utils::{keybindings, layer_window, LayerOptions},
};
use gtk4::{gio::Cancellable, prelude::GtkWindowExt, Application, Window};
use vte4::TerminalExtManual;

pub(crate) struct Terminal;

impl Terminal {
    pub(crate) fn activate(app: &Application) {
        let terminal = vte4::Terminal::builder().build();

        terminal.spawn_async(
            vte4::PtyFlags::DEFAULT,
            Some(&std::env::var("HOME").unwrap()),
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

        let window = load_widget::<Window>("Htop");
        window.set_application(Some(app));
        window.set_child(Some(&terminal));

        layer_window(
            window,
            LayerOptions::builder()
                .with_namespace("Htop")
                .with_layer(gtk4_layer_shell::Layer::Overlay)
                .with_anchors(&[gtk4_layer_shell::Edge::Top, gtk4_layer_shell::Edge::Right])
                .with_margins(&[
                    (gtk4_layer_shell::Edge::Top, 50),
                    (gtk4_layer_shell::Edge::Right, 600),
                ])
                .with_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive)
                .build(),
        );

        keybindings(window)
            .add("Escape", || window.close())
            .fallback(|_key| {})
            .finish();

        GlobalWindows::set_reset_fn("Htop", || {});
    }
}
