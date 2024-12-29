use crate::ffi::{CArray, CString};
use crate::modules::weather::WeatherCode;

#[derive(Debug)]
#[repr(C)]
pub enum Event {
    Memory {
        used: f64,
        total: f64,
    },
    CpuUsage {
        usage_per_core: CArray<usize>,
    },
    Time {
        date: CString,
        time: CString,
    },
    Workspaces {
        ids: CArray<usize>,
        active_id: usize,
    },
    Language {
        lang: CString,
    },
    AppList {
        apps: CArray<App>,
    },
    Volume {
        volume: f32,
    },
    CurrentWeather {
        temperature: f32,
        code: WeatherCode,
    },
    ForecastWeather {
        hourly: CArray<WeatherOnHour>,
        daily: CArray<WeatherOnDay>,
    },
    WiFiStatus {
        ssid: CString,
        strength: u8,
    },
    NetworkList {
        list: CArray<Network>,
    },
    ToggleLauncher,
    ToggleSessionScreen,
}

#[derive(Debug)]
#[repr(C)]
pub struct App {
    pub name: CString,
    pub selected: bool,
    pub icon: AppIcon,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub enum AppIcon {
    IconPath(CString),
    IconName(CString),
}

#[derive(Debug)]
#[repr(C)]
pub struct WeatherOnHour {
    pub hour: CString,
    pub temperature: f32,
    pub code: WeatherCode,
}

#[derive(Debug)]
#[repr(C)]
pub struct WeatherOnDay {
    pub day: CString,
    pub temperature_min: f32,
    pub temperature_max: f32,
    pub code: WeatherCode,
}

#[derive(Debug)]
#[repr(C)]
pub struct Network {
    pub iface: CString,
    pub address: CString,
}
