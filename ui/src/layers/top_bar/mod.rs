use crate::widgets::top_bar::TopBarWindow;
use gtk4::{prelude::GtkWindowExt, Application};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

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

impl TopBar {
    pub(crate) fn activate(app: &Application) {
        let window = TopBarWindow();

        window.set_application(Some(app));

        LayerShell::init_layer_shell(window);
        LayerShell::set_layer(window, Layer::Top);
        LayerShell::set_anchor(window, Edge::Top, true);
        LayerShell::set_anchor(window, Edge::Left, true);
        LayerShell::set_anchor(window, Edge::Right, true);
        LayerShell::set_margin(window, Edge::Top, 0);
        LayerShell::set_namespace(window, "LayerShell/TopBar");

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
