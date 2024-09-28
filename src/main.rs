#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::missing_transmute_annotations)]
#![allow(clippy::type_complexity)]

mod ffi;
mod globals;
mod layers;
mod models;
mod utils;

use gtk4::{
    prelude::{ApplicationExt, ApplicationExtManual},
    Application,
};

use crate::{
    globals::GlobalWidgets,
    layers::{Htop, Launcher, LogoutScreen, Networks, TopBar, Weather},
    models::{NetworkList, OutputSound, WeatherApi},
    utils::{load_css, parse_args, IPC},
};

const APP_ID: &str = "com.me.LayerShell";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    parse_args()?;
    IPC::spawn()?;

    WeatherApi::spawn();
    OutputSound::spawn();
    models::init();

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(|app| {
        GlobalWidgets::init();
        NetworkList::spawn();

        TopBar::activate(app);
        LogoutScreen::activate(app);
        Launcher::activate(app);
        Networks::activate(app);
        Htop::activate(app);
        Weather::activate(app);

        models::spawn_all();
    });

    app.connect_startup(|_app| {
        load_css();
    });

    app.run_with_args(&[""]);

    Ok(())
}
