use crate::utils::{singleton, LayerWindow};
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
singleton!(TopBar);

impl LayerWindow for TopBar {
    const NAME: &str = "TopBar";
    const LAYER: Layer = Layer::Overlay;
    const ANCHORS: &[Edge] = &[Edge::Top, Edge::Left, Edge::Right];
    const MARGINS: &[(Edge, i32)] = &[(Edge::Top, 0)];
    const KEYBOARD_MODE: Option<KeyboardMode> = None;

    fn reset(&self) {}
}

impl TopBar {
    pub(crate) fn activate(app: &Application) {
        let window = Self::layer_window(app);

        workspaces::init(5);
        htop::init();
        language::init();
        sound::init();
        cpu::init();
        ram::init();
        network::init();
        clock::init("%H:%M:%S", "%Y %B %e\n%A");
        power::init();
        weather::init();

        window.present();
    }
}
