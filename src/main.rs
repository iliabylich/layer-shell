#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::missing_transmute_annotations)]

mod ffi;
mod globals;
mod layers;
mod models;
mod utils;
mod widgets;
mod windows;

use gtk4::{
    prelude::{ApplicationExt, ApplicationExtManual},
    Application,
};

use crate::{
    globals::{GlobalWidgets, GlobalWindows},
    layers::{Htop, Launcher, LogoutScreen, Networks, TopBar},
    models::{NetworkList, WeatherApi},
    utils::{load_css, parse_args, HyprlandClient, IPC},
    windows::Weather,
};

const APP_ID: &str = "com.me.LayerShell";

fn main() {
    parse_args();
    IPC::subscribe();

    HyprlandClient::start();
    GlobalWindows::init();
    WeatherApi::spawn();
    NetworkList::spawn_once();

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(|app| {
        GlobalWidgets::init();
        TopBar::activate(app);
        LogoutScreen::activate(app);
        Launcher::activate(app);
        Networks::activate(app);
        Htop::activate(app);
        Weather::activate(app);
    });

    app.connect_startup(|_app| {
        load_css();
    });

    app.run_with_args(&[""]);
}
