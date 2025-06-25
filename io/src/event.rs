use ffi::{CArray, COption, CString};
use weather::WeatherCode;

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
        code: WeatherCode,
    },
    ForecastWeather {
        hourly: CArray<WeatherOnHour>,
        daily: CArray<WeatherOnDay>,
    },
    WifiStatus {
        wifi_status: COption<WifiStatus>,
    },
    UploadSpeed {
        speed: CString,
    },
    DownloadSpeed {
        speed: CString,
    },
    NetworkList {
        list: CArray<Network>,
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

impl From<clock::Event> for Event {
    fn from(event: clock::Event) -> Self {
        Self::Time {
            time: event.time.into(),
        }
    }
}

impl From<control::Event> for Event {
    fn from(event: control::Event) -> Self {
        match event {
            control::Event::ToggleSessionScreen => Self::ToggleSessionScreen,
            control::Event::ReloadStyles => Self::ReloadStyles,
            control::Event::Exit => Self::Exit,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Network {
    pub iface: CString,
    pub address: CString,
}

impl From<network::NetworkData> for Network {
    fn from(input: network::NetworkData) -> Self {
        Self {
            iface: input.iface.into(),
            address: input.address.into(),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct WifiStatus {
    pub ssid: CString,
    pub strength: u8,
}

impl From<network::WifiStatus> for WifiStatus {
    fn from(input: network::WifiStatus) -> Self {
        Self {
            ssid: input.ssid.into(),
            strength: input.strength,
        }
    }
}

impl From<network::Event> for Event {
    fn from(event: network::Event) -> Self {
        match event {
            network::Event::WifiStatus { wifi_status } => Self::WifiStatus {
                wifi_status: wifi_status.into(),
            },
            network::Event::UploadSpeed { speed } => Self::UploadSpeed {
                speed: speed.into(),
            },
            network::Event::DownloadSpeed { speed } => Self::DownloadSpeed {
                speed: speed.into(),
            },
            network::Event::NetworkList { list } => Self::NetworkList { list: list.into() },
        }
    }
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

impl From<weather::WeatherOnHour> for WeatherOnHour {
    fn from(input: weather::WeatherOnHour) -> Self {
        Self {
            hour: input.hour.into(),
            temperature: input.temperature,
            code: input.code,
        }
    }
}

impl From<weather::WeatherOnDay> for WeatherOnDay {
    fn from(input: weather::WeatherOnDay) -> Self {
        Self {
            day: input.day.into(),
            temperature_min: input.temperature_min,
            temperature_max: input.temperature_max,
            code: input.code,
        }
    }
}

impl From<weather::Event> for Event {
    fn from(event: weather::Event) -> Self {
        match event {
            weather::Event::CurrentWeather { temperature, code } => {
                Self::CurrentWeather { temperature, code }
            }
            weather::Event::ForecastWeather { hourly, daily } => Self::ForecastWeather {
                hourly: hourly.into(),
                daily: daily.into(),
            },
        }
    }
}
