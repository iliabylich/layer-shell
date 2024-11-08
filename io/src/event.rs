use std::collections::HashSet;

#[derive(Debug, Clone)]
pub enum Event {
    Memory {
        used: f64,
        total: f64,
    },
    Cpu {
        usage_per_core: Vec<usize>,
    },
    Time {
        time: String,
        date: String,
    },
    Workspaces {
        ids: HashSet<usize>,
        active_id: usize,
    },
    Language {
        lang: String,
    },
    AppList(Vec<App>),
    Volume(f64),
    WeatherCurrent(String),
    WeatherForecast {
        hourly: Vec<(String, String)>,
        daily: Vec<(String, String)>,
    },
    WiFi(Option<(String, u8)>),
    NetworkList(Vec<(String, String)>),
    ToggleLauncher,
    ToggleLogoutScreen,
}

#[derive(Debug, Clone)]
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
