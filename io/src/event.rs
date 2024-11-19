use crate::weather::WeatherCode;
use std::collections::HashSet;

#[derive(Debug)]
pub enum Event {
    Memory {
        used: f64,
        total: f64,
    },
    Cpu(Vec<usize>),
    Time {
        time: String,
        date: String,
    },
    Workspaces {
        ids: HashSet<usize>,
        active_id: usize,
    },
    Language(String),
    AppList(Vec<App>),
    Volume(f64),
    WeatherCurrent {
        temperature: f32,
        code: WeatherCode,
    },
    WeatherForecast {
        hourly: Vec<WeatherOnHour>,
        daily: Vec<WeatherOnDay>,
    },
    WiFiStatus(Option<WiFiStatus>),
    NetworkList(Vec<Network>),
    ToggleLauncher,
    ToggleLogoutScreen,
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
pub struct WeatherOnHour {
    pub hour: String,
    pub temperature: f32,
    pub code: WeatherCode,
}

#[derive(Debug)]
pub struct WeatherOnDay {
    pub day: String,
    pub temperature: std::ops::Range<f32>,
    pub code: WeatherCode,
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
