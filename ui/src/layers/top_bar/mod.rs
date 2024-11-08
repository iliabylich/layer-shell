use crate::{utils::LayerWindow, widgets::TopBarWindow};
use gtk4::{prelude::GtkWindowExt, Application};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer};

mod clock;
mod cpu;
mod htop;
mod language;
mod network;
mod power;
mod ram;
mod sound;
mod weather;
mod workspaces;

pub(crate) struct TopBar;

impl LayerWindow for TopBar {
    const NAME: &'static str = "TopBar";
    const LAYER: Layer = Layer::Top;
    const ANCHORS: &'static [Edge] = &[Edge::Top, Edge::Left, Edge::Right];
    const MARGINS: &'static [(Edge, i32)] = &[(Edge::Top, 0)];
    const KEYBOARD_MODE: Option<KeyboardMode> = None;

    fn reset() {}

    fn window() -> &'static gtk4::Window {
        TopBarWindow()
    }
}

impl TopBar {
    pub(crate) fn activate(app: &Application) {
        let window = Self::layer_window(app);

        workspaces::init();
        htop::init();
        language::init();
        sound::init();
        cpu::init();
        ram::init();
        network::init();
        clock::init();
        power::init();
        weather::init();

        window.present();
    }
}
