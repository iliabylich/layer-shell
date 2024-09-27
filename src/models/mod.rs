mod hyprland;
pub(crate) use hyprland::{HyprlandLanguage, HyprlandWorkspaces};

mod cpu;
pub(crate) use cpu::CPU;

mod memory;
pub(crate) use memory::{Memory, MemoryData};

mod time;
pub(crate) use time::Time;

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

pub(crate) fn spawn_all() {
    std::thread::spawn(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();

        rt.block_on(async {
            tokio::join!(Memory::spawn());
        });
    });
}
