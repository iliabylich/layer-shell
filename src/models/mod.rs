mod cpu;
mod hyprland;
mod memory;
mod time;

mod output_sound;
pub(crate) use output_sound::OutputSound;

mod logout;
pub(crate) use logout::Logout;

mod network_manager;
pub(crate) use network_manager::{NetworkList, WiFiStatus};

mod app_list;
pub(crate) use app_list::AppList;

mod weather_api;
use tokio::sync::mpsc::Sender;
pub(crate) use weather_api::WeatherApi;

mod event;
pub(crate) use event::Event;

mod command;
pub(crate) use command::Command;

struct Model {
    subscriptions: Vec<fn(&Event)>,
    commander: Sender<Command>,
}
crate::utils::singleton!(Model);

pub(crate) fn spawn_all() {
    let (etx, mut erx) = tokio::sync::mpsc::channel::<Event>(100);
    let (ctx, crx) = tokio::sync::mpsc::channel::<Command>(100);

    Model::set(Model {
        subscriptions: vec![],
        commander: ctx,
    });

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .enable_io()
            .build()
            .unwrap();

        rt.block_on(async {
            tokio::join!(
                command::start_processing(crx),
                memory::spawn(etx.clone()),
                cpu::spawn(etx.clone()),
                time::spawn(etx.clone()),
                hyprland::spawn(etx.clone()),
            );
        });
    });

    gtk4::glib::spawn_future_local(async move {
        while let Some(event) = erx.recv().await {
            for f in Model::get().subscriptions.iter() {
                (f)(&event);
            }
        }
    });
}

pub(crate) fn subscribe(f: fn(&Event)) {
    Model::get().subscriptions.push(f);
}

pub(crate) fn publish(c: Command) {
    gtk4::glib::spawn_future_local(async move {
        Model::get().commander.send(c).await.unwrap();
    });
}
