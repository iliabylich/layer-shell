mod app_list;
mod command;
mod cpu;
mod event;
mod hyprland;
mod memory;
mod network_manager;
mod output_sound;
mod session;
mod subscriptions;
mod time;
mod weather;

pub use command::Command;
pub use event::{App, AppIcon, Event};
pub use subscriptions::subscribe;

layer_shell_utils::global!(COMMANDER, tokio::sync::mpsc::Sender<Command>);

pub fn spawn_all() {
    pretty_env_logger::init();

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
                network_manager::network_list::spawn(etx.clone()),
            );
        });
    });

    glib::spawn_future_local(async move {
        while let Some(event) = erx.recv().await {
            log::info!("Received event {:?}", event);

            for f in subscriptions::all().iter() {
                (f)(&event);
            }
        }
    });
}

pub fn publish(c: Command) {
    glib::spawn_future_local(async move {
        if let Err(err) = COMMANDER::get().send(c).await {
            log::error!("failed to publish event: {}", err);
        }
    });
}

pub fn init() {
    subscriptions::init();
}
