use layer_shell_hyprland::{Language, Workspaces};
use layer_shell_pipewire::Volume;
use layer_shell_weather::{CurrentWeather, ForecastWeather};

#[derive(Debug)]
pub enum Event {
    Memory { used: f64, total: f64 },
    Cpu(Vec<usize>),
    Time { time: String, date: String },
    Workspaces(Workspaces),
    Language(Language),
    AppList(Vec<App>),
    Volume(Volume),
    CurrentWeather(CurrentWeather),
    ForecastWeather(ForecastWeather),
    WiFiStatus(Option<WiFiStatus>),
    NetworkList(Vec<Network>),
    ToggleLauncher,
    ToggleSessionScreen,
}

#[derive(Debug)]
pub struct App {
    pub name: String,
    pub selected: bool,
    pub icon: AppIcon,
}
#[derive(Debug, Clone)]
pub enum AppIcon {
    IconPath(String),
    IconName(String),
}

#[derive(Debug)]
pub struct WiFiStatus {
    pub ssid: String,
    pub strength: u8,
}

#[derive(Debug)]
pub struct Network {
    pub iface: String,
    pub address: String,
}
