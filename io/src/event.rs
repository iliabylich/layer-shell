use layer_shell_app_list::AppList;
use layer_shell_hyprland::{Language, Workspaces};
use layer_shell_memory::Memory;
use layer_shell_network::{NetworkList, WiFiStatus};
use layer_shell_pipewire::Volume;
use layer_shell_weather::{CurrentWeather, ForecastWeather};

#[derive(Debug)]
pub enum Event {
    Memory(Memory),
    Cpu(Vec<usize>),
    Time { time: String, date: String },
    Workspaces(Workspaces),
    Language(Language),
    AppList(AppList),
    Volume(Volume),
    CurrentWeather(CurrentWeather),
    ForecastWeather(ForecastWeather),
    WiFiStatus(Option<WiFiStatus>),
    NetworkList(NetworkList),
    ToggleLauncher,
    ToggleSessionScreen,
}
