use crate::{
    ffi::{CArray, COption, CString},
    modules::weather::WeatherCode,
};

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
        time: CString,
    },
    Workspaces {
        ids: CArray<usize>,
        active_id: usize,
    },
    Language {
        lang: CString,
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
        apps: CArray<TrayApp>,
    },
    ToggleSessionScreen,
    ReloadStyles,
    Exit,
}

unsafe impl Sync for Event {}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct WeatherOnHour {
    pub hour: CString,
    pub temperature: f32,
    pub code: WeatherCode,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct WeatherOnDay {
    pub day: CString,
    pub temperature_min: f32,
    pub temperature_max: f32,
    pub code: WeatherCode,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Network {
    pub iface: CString,
    pub address: CString,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct WifiStatus {
    pub ssid: CString,
    pub strength: u8,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct TrayApp {
    pub root_item: TrayItem,
    pub icon: TrayIcon,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct TrayItem {
    pub id: i32,
    pub uuid: CString,
    pub type_: CString,
    pub label: CString,
    pub enabled: bool,
    pub visible: bool,
    pub icon_name: CString,
    pub icon_data: CString,
    pub toggle_type: CString,
    pub toggle_state: i64,
    pub children_display: CString,
    pub children: CArray<TrayItem>,
}

#[derive(Clone)]
#[repr(C)]
pub enum TrayIcon {
    Path { path: CString },
    Name { name: CString },
    PixmapVariant { w: u32, h: u32, bytes: CArray<u8> },
    Unset,
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
            TrayIcon::Unset => write!(f, "None"),
        }
    }
}
