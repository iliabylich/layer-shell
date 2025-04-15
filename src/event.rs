use crate::modules::weather::WeatherCode;
use pyo3::pyclass;

#[derive(Debug, Clone)]
#[pyclass]
#[must_use]
pub(crate) enum Event {
    Memory {
        used: f64,
        total: f64,
    },
    CpuUsage {
        usage_per_core: Vec<usize>,
    },
    Time {
        time: String,
    },
    Workspaces {
        ids: Vec<usize>,
        active_id: usize,
    },
    Language {
        lang: String,
    },
    Launcher {
        apps: Vec<LauncherApp>,
    },
    Volume {
        volume: u32,
        muted: bool,
    },
    CurrentWeather {
        temperature: f32,
        code: WeatherCode,
    },
    ForecastWeather {
        hourly: Vec<WeatherOnHour>,
        daily: Vec<WeatherOnDay>,
    },
    WifiStatus {
        wifi_status: Option<WifiStatus>,
    },
    NetworkSpeed {
        upload_speed: String,
        download_speed: String,
    },
    NetworkList {
        list: Vec<Network>,
    },
    Tray {
        apps: Vec<TrayApp>,
    },
    ToggleLauncher(),
    ToggleSessionScreen(),
    ReloadStyles(),
}

unsafe impl Sync for Event {}

#[derive(Debug, Clone)]
#[pyclass]
pub(crate) struct LauncherApp {
    #[pyo3(get)]
    pub(crate) name: String,
    #[pyo3(get)]
    pub(crate) selected: bool,
    #[pyo3(get)]
    pub(crate) icon: LauncherAppIcon,
}

#[derive(Debug, Clone)]
#[pyclass]
pub(crate) enum LauncherAppIcon {
    IconPath(String),
    IconName(String),
}

#[derive(Debug, Clone)]
#[pyclass]
pub(crate) struct WeatherOnHour {
    #[pyo3(get)]
    pub(crate) hour: String,
    #[pyo3(get)]
    pub(crate) temperature: f32,
    #[pyo3(get)]
    pub(crate) code: WeatherCode,
}

#[derive(Debug, Clone)]
#[pyclass]
pub(crate) struct WeatherOnDay {
    #[pyo3(get)]
    pub(crate) day: String,
    #[pyo3(get)]
    pub(crate) temperature_min: f32,
    #[pyo3(get)]
    pub(crate) temperature_max: f32,
    #[pyo3(get)]
    pub(crate) code: WeatherCode,
}

#[derive(Debug, Clone)]
#[pyclass]
pub(crate) struct Network {
    #[pyo3(get)]
    pub(crate) iface: String,
    #[pyo3(get)]
    pub(crate) address: String,
}

#[derive(Debug, Clone)]
#[pyclass]
pub(crate) struct WifiStatus {
    #[pyo3(get)]
    pub(crate) ssid: String,
    #[pyo3(get)]
    pub(crate) strength: u8,
}

#[derive(Debug, Clone)]
#[pyclass]
pub(crate) struct TrayApp {
    #[pyo3(get)]
    pub(crate) root_item: TrayItem,
    #[pyo3(get)]
    pub(crate) icon: TrayIcon,
}

#[derive(Debug, Clone)]
#[pyclass]
pub(crate) struct TrayItem {
    #[pyo3(get)]
    pub(crate) id: i32,
    #[pyo3(get)]
    pub(crate) uuid: String,
    #[pyo3(get)]
    pub(crate) type_: String,
    #[pyo3(get)]
    pub(crate) label: String,
    #[pyo3(get)]
    pub(crate) enabled: bool,
    #[pyo3(get)]
    pub(crate) visible: bool,
    #[pyo3(get)]
    pub(crate) icon_name: String,
    #[pyo3(get)]
    pub(crate) icon_data: String,
    #[pyo3(get)]
    pub(crate) toggle_type: String,
    #[pyo3(get)]
    pub(crate) toggle_state: i64,
    #[pyo3(get)]
    pub(crate) children_display: String,
    #[pyo3(get)]
    pub(crate) children: Vec<TrayItem>,
}

#[derive(Clone)]
#[pyclass]
pub(crate) enum TrayIcon {
    Path { path: String },
    Name { name: String },
    PixmapVariant { w: u32, h: u32, bytes: Vec<u8> },
    Unset(),
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
                .field("bytes", &format!("[...{} bytes]", bytes.len()))
                .finish(),
            TrayIcon::Unset() => write!(f, "None"),
        }
    }
}
