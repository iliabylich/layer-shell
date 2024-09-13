use gtk4::{
    prelude::{ApplicationExt, ApplicationExtManual},
    Application,
};

mod globals;
mod models;
mod utils;
mod widgets;
mod windows;

use windows::{Launcher, LogoutScreen, Networks, TopBar};

const APP_ID: &str = "com.me.layershell";

fn main() {
    utils::HyprlandClient::start();

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(|app| {
        globals::GlobalWidgets::init();
        TopBar::activate(app);
        LogoutScreen::activate(app);
        Launcher::activate(app);
        Networks::activate(app);
        // Terminal::activate();
    });

    app.connect_startup(|_app| {
        utils::load_css();
    });

    let args: &[&str] = &[];
    app.run_with_args(args);
}
