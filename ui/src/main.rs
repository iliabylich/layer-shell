#![allow(clippy::type_complexity)]

mod icons;
mod layers;
mod utils;
mod widgets;

use gtk4::{
    prelude::{ApplicationExt, ApplicationExtManual},
    Application,
};

use crate::{
    layers::{Htop, Launcher, Networks, SessionScreen, TopBar, Weather},
    utils::load_css,
};

fn main() {
    pretty_env_logger::init();
    layer_shell_io::init();

    gtk4::glib::unix_signal_add(10 /* USR1 */, || {
        layer_shell_io::on_sigusr1();
        gtk4::glib::ControlFlow::Continue
    });

    let app = Application::builder()
        .application_id("com.me.LayerShell")
        .build();

    app.connect_activate(|app| {
        icons::load();
        widgets::load();

        TopBar::activate(app);
        SessionScreen::activate(app);
        Launcher::activate(app);
        Networks::activate(app);
        Htop::activate(app);
        Weather::activate(app);

        layer_shell_io::spawn_thread();

        gtk4::glib::timeout_add(std::time::Duration::from_millis(50), || {
            layer_shell_io::poll_events();
            gtk4::glib::ControlFlow::Continue
        });
    });

    app.connect_startup(|_app| {
        if let Err(err) = load_css() {
            log::error!("Failed to load css: {:?}", err);
            std::process::exit(1);
        }
    });

    app.run_with_args(&[""]);
}
