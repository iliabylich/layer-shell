use crate::{
    globals::load_widget,
    utils::{keybindings, layer_window, singleton, LayerOptions, ToggleWindow},
};
use gtk4::{gio::Cancellable, prelude::GtkWindowExt, Application, Window};
use vte4::TerminalExtManual;

pub(crate) struct Htop {
    reset: Box<dyn Fn()>,
}
singleton!(Htop);

impl Htop {
    pub(crate) fn activate(app: &Application) {
        let window = load_widget::<Window>("Htop");
        window.set_application(Some(app));

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
        window.set_child(Some(&terminal));

        keybindings(window)
            .add("Escape", || Self::toggle())
            .fallback(|_key| {})
            .finish();

        let reset = Box::new(|| {});

        Self::set(Self { reset })
    }
}

impl ToggleWindow for Htop {
    fn reset(&self) {
        (self.reset)()
    }

    fn window(&self) -> &'static Window {
        load_widget::<Window>("Htop")
    }
}
