use std::time::Duration;

use layer_shell_utils::global;

mod app_list;
mod command;
mod cpu;
mod event;
mod hyprland;
mod memory;
mod network_manager;
mod output_sound;
mod session;
mod time;
mod weather;

pub use command::Command;
pub use event::{App, AppIcon, Event};

global!(COMMANDER, std::sync::mpsc::Sender<Command>);
global!(SUBSCRIPTIONS, Vec<fn(&Event)>);

pub fn subscribe(f: fn(&Event)) {
    SUBSCRIPTIONS::get().push(f);
}

pub fn init() {
    pretty_env_logger::init();
    SUBSCRIPTIONS::set(vec![]);
}

pub fn spawn_all() {
    let (etx, erx) = std::sync::mpsc::channel::<Event>();
    let (ctx, crx) = std::sync::mpsc::channel::<Command>();

    COMMANDER::set(ctx);

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .enable_io()
            .build()
            .unwrap();

        rt.block_on(async {
            tokio::join!(
                // command processing actor
                command::start_processing(crx),
                // and all models
                memory::spawn(etx.clone()),
                cpu::spawn(etx.clone()),
                time::spawn(etx.clone()),
                hyprland::spawn(etx.clone()),
                app_list::spawn(etx.clone()),
                output_sound::spawn(etx.clone()),
                session::spawn(etx.clone()),
                weather::spawn(etx.clone()),
                network_manager::wifi_status::spawn(etx.clone()),
                network_manager::network_list::spawn(etx.clone()),
            );
        });
    });

    glib::timeout_add(Duration::from_millis(50), move || {
        while let Ok(event) = erx.try_recv() {
            log::info!("Received event {:?}", event);

            for f in SUBSCRIPTIONS::get().iter() {
                (f)(&event);
            }
        }
        return glib::ControlFlow::Continue;
    });
}

pub fn publish(c: Command) {
    if let Err(err) = COMMANDER::get().send(c) {
        log::error!("failed to publish event: {}", err);
    }
}
