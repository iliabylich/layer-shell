use crate::ffi::{CArray, COption, CString};
use crate::modules::weather::WeatherCode;

#[derive(Debug)]
#[repr(C)]
#[must_use]
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
    Launcher {
        apps: CArray<App>,
    },
    Volume {
        volume: u32,
        muted: bool,
    },
    Mute {
        muted: bool,
    },
    CurrentWeather {
        temperature: f32,
        code: WeatherCode,
    },
    ForecastWeather {
        hourly: CArray<WeatherOnHour>,
        daily: CArray<WeatherOnDay>,
    },
    WifiStatus {
        wifi_status: COption<WifiStatus>,
    },
    NetworkSpeed {
        upload_speed: CString,
        download_speed: CString,
    },
    NetworkList {
        list: CArray<Network>,
    },
    Tray {
        list: CArray<TrayApp>,
    },
    ToggleLauncher,
    ToggleSessionScreen,
}

unsafe impl Sync for Event {}

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

#[derive(Debug)]
#[repr(C)]
pub struct WifiStatus {
    pub ssid: CString,
    pub strength: u8,
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct TrayApp {
    pub items: CArray<TrayItem>,
    pub icon: TrayIcon,
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct TrayItem {
    pub label: CString,
    pub disabled: bool,
    pub uuid: CString,
}

#[derive(Clone, PartialEq)]
#[repr(C)]
pub enum TrayIcon {
    Path { path: CString },
    Name { name: CString },
    PixmapVariant { w: u32, h: u32, bytes: CArray<u8> },
    None,
}

impl std::fmt::Debug for TrayIcon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrayIcon::Path { path } => f.debug_struct("TrayIconPath").field("path", path).finish(),
            TrayIcon::Name { name } => f.debug_struct("TrayIconName").field("name", name).finish(),
            TrayIcon::PixmapVariant { w, h, bytes } => f
                .debug_struct("TrayIconPixmapVariant")
                .field("w", w)
                .field("h", h)
                .field("bytes", &format!("[...{} bytes]", bytes.len))
                .finish(),
            TrayIcon::None => write!(f, "None"),
        }
    }
}
