use ffi::{CArray, CString};

#[derive(Debug)]
#[repr(C)]
#[must_use]
pub enum Event {
    Memory {
        used: f64,
        total: f64,
    },
    CpuUsage {
        usage_per_core: CArray<u8>,
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
        // code: WeatherCode,
    },
    ForecastWeather {
        // hourly: CArray<WeatherOnHour>,
        // daily: CArray<WeatherOnDay>,
    },
    WifiStatus {
        // wifi_status: COption<WifiStatus>,
    },
    NetworkSpeed {
        upload_speed: CString,
        download_speed: CString,
    },
    NetworkList {
        // list: CArray<Network>,
    },
    Tray {
        // apps: CArray<TrayApp>,
    },
    ToggleSessionScreen,
    ReloadStyles,
    Exit,
}

impl From<hyprland::Event> for Event {
    fn from(event: hyprland::Event) -> Self {
        match event {
            hyprland::Event::Workspaces { ids, active_id } => Self::Workspaces {
                ids: ids.into(),
                active_id,
            },
            hyprland::Event::Language { lang } => Self::Language { lang: lang.into() },
        }
    }
}

impl From<cpu::Event> for Event {
    fn from(event: cpu::Event) -> Self {
        Self::CpuUsage {
            usage_per_core: event.usage_per_core.into(),
        }
    }
}

impl From<memory::Event> for Event {
    fn from(event: memory::Event) -> Self {
        Self::Memory {
            used: event.used,
            total: event.total,
        }
    }
}
