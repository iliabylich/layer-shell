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
    layers::{Htop, Launcher, LogoutScreen, Networks, TopBar, Weather},
    utils::load_css,
};

const APP_ID: &str = "com.me.LayerShell";

fn main() {
    gtk4::glib::unix_signal_add(10 /* USR1 */, move || {
        layer_shell_io::on_sigusr1();
        gtk4::glib::ControlFlow::Continue
    });

    layer_shell_io::init();

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(|app| {
        widgets::load();
        icons::load();

        TopBar::activate(app);
        LogoutScreen::activate(app);
        Launcher::activate(app);
        Networks::activate(app);
        Htop::activate(app);
        Weather::activate(app);

        layer_shell_io::spawn_all();

        gtk4::glib::timeout_add(std::time::Duration::from_millis(50), || {
            layer_shell_io::poll_events();
            gtk4::glib::ControlFlow::Continue
        });
    });

    app.connect_startup(|_app| {
        if let Err(err) = load_css() {
            eprintln!("Failed to load css: {:?}", err);
            std::process::exit(1);
        }
    });

    app.run_with_args(&[""]);
}
