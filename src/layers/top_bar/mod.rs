use crate::{
    globals::load_widget,
    utils::{layer_window, LayerOptions},
};
use gtk4::{prelude::GtkWindowExt, Application, Window};

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
    const NAME: &str = "TopBar";

    pub(crate) fn activate(app: &Application) {
        let window = load_widget::<Window>(Self::NAME);
        window.set_application(Some(app));
        layer_window(
            window,
            LayerOptions::builder()
                .with_namespace(Self::NAME)
                .with_layer(gtk4_layer_shell::Layer::Overlay)
                .with_anchors(&[
                    gtk4_layer_shell::Edge::Top,
                    gtk4_layer_shell::Edge::Left,
                    gtk4_layer_shell::Edge::Right,
                ])
                .with_margins(&[(gtk4_layer_shell::Edge::Top, 0)])
                .build(),
        );

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
