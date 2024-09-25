use crate::utils::{keybindings, singleton, LayerWindow};
use gtk4::{gio::Cancellable, prelude::GtkWindowExt, Application};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer};
use vte4::TerminalExtManual;

pub(crate) struct Htop {
    reset: Box<dyn Fn()>,
}
singleton!(Htop);

impl LayerWindow for Htop {
    const NAME: &'static str = "Htop";
    const LAYER: Layer = Layer::Overlay;
    const ANCHORS: &'static [Edge] = &[Edge::Top, Edge::Right];
    const MARGINS: &'static [(Edge, i32)] = &[(Edge::Top, 50), (Edge::Right, 600)];
    const KEYBOARD_MODE: Option<KeyboardMode> = Some(KeyboardMode::Exclusive);

    fn reset(&self) {
        (self.reset)()
    }
}

impl Htop {
    pub(crate) fn activate(app: &Application) {
        let window = Self::layer_window(app);

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
            .add("Escape", Self::toggle)
            .fallback(|_key| {})
            .finish();

        let reset = Box::new(|| {});

        Self::set(Self { reset })
    }
}
