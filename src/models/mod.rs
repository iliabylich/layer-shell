mod app_list;
mod cpu;
mod hyprland;
mod memory;
mod output_sound;
mod session;
mod time;
mod weather;

mod network_manager;
pub(crate) use network_manager::NetworkList;

mod event;
pub(crate) use event::{App, AppIcon, Event};

mod command;
pub(crate) use command::Command;

mod subscriptions;
pub(crate) use subscriptions::subscribe;

use crate::utils::global;

global!(COMMANDER, tokio::sync::mpsc::Sender<Command>);

pub(crate) fn spawn_all() {
    let (etx, mut erx) = tokio::sync::mpsc::channel::<Event>(100);
    let (ctx, crx) = tokio::sync::mpsc::channel::<Command>(100);

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
            );
        });
    });

    gtk4::glib::spawn_future_local(async move {
        while let Some(event) = erx.recv().await {
            log::info!("Received event {:?}", event);

            for f in subscriptions::all().iter() {
                (f)(&event);
            }
        }
    });
}

pub(crate) fn publish(c: Command) {
    gtk4::glib::spawn_future_local(async move {
        if let Err(err) = COMMANDER::get().send(c).await {
            log::error!("failed to publish event: {}", err);
        }
    });
}

pub(crate) fn init() {
    subscriptions::init();
}
