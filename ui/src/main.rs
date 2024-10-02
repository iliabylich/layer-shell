#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::missing_transmute_annotations)]
#![allow(clippy::type_complexity)]

mod globals;
mod layers;
mod utils;

use gtk4::{
    prelude::{ApplicationExt, ApplicationExtManual},
    Application,
};

use crate::{
    globals::GlobalWidgets,
    layers::{Htop, Launcher, LogoutScreen, Networks, TopBar, Weather},
    utils::{load_css, parse_args, IPC},
};

const APP_ID: &str = "com.me.LayerShell";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    parse_args()?;
    IPC::spawn()?;

    layer_shell_io::init();

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(|app| {
        GlobalWidgets::init();

        TopBar::activate(app);
        LogoutScreen::activate(app);
        Launcher::activate(app);
        Networks::activate(app);
        Htop::activate(app);
        Weather::activate(app);

        layer_shell_io::spawn_all();
    });

    app.connect_startup(|_app| {
        load_css();
    });

    app.run_with_args(&[""]);

    Ok(())
}
