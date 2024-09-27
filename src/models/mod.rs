mod cpu;
mod memory;
mod time;

mod hyprland;
pub(crate) use hyprland::{HyprlandLanguage, HyprlandWorkspaces};

mod output_sound;
pub(crate) use output_sound::OutputSound;

mod logout;
pub(crate) use logout::Logout;

mod network_manager;
pub(crate) use network_manager::{NetworkList, WiFiStatus};

mod app_list;
pub(crate) use app_list::AppList;

mod weather_api;
pub(crate) use weather_api::WeatherApi;

mod event;
pub(crate) use event::Event;

struct Model {
    subscriptions: Vec<fn(&Event)>,
}
crate::utils::singleton!(Model);

pub(crate) fn spawn_all() {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Event>(100);
    Model::set(Model {
        subscriptions: vec![],
    });

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();

        rt.block_on(async {
            tokio::join!(
                memory::spawn(tx.clone()),
                cpu::spawn(tx.clone()),
                time::spawn(tx.clone())
            );
        });
    });

    gtk4::glib::spawn_future_local(async move {
        while let Some(event) = rx.recv().await {
            for f in Model::get().subscriptions.iter() {
                (f)(&event);
            }
        }
    });
}

pub(crate) fn subscribe(f: fn(&Event)) {
    Model::get().subscriptions.push(f);
}
