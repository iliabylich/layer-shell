use crate::{
    ffi::FFIArray,
    modules::{
        DAILY_WEATHER_FORECAST_LENGTH, HOURLY_WEATHER_FORECAST_LENGTH, KbModKind, TrayMenu,
        WeatherCode, WeatherOnDay, WeatherOnHour,
    },
    utils::StringRef,
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
        usage_per_core: FFIArray<u8>,
    },
    Time {
        now: StringRef,
    },
    Language {
        lang: StringRef,
    },
    Weather {
        temperature: f32,
        code: WeatherCode,
        hourly_forecast: [WeatherOnHour; HOURLY_WEATHER_FORECAST_LENGTH],
        daily_forecast: [WeatherOnDay; DAILY_WEATHER_FORECAST_LENGTH],
    },
    NetworkSsidAndStrength {
        ssid: StringRef,
        strength: u8,
    },
    UploadSpeed {
        bytes_per_sec: u64,
    },
    DownloadSpeed {
        bytes_per_sec: u64,
    },
    TrayAppAdded {
        service: u32,
        menu: TrayMenu,
        icon: StringRef,
    },
    TrayAppIconUpdated {
        service: u32,
        icon: StringRef,
    },
    TrayAppMenuUpdated {
        service: u32,
        menu: TrayMenu,
    },
    TrayAppRemoved {
        service: u32,
    },
    ToggleSessionScreen,
    KbModToggled {
        kind: KbModKind,
        enabled: bool,
    },
    Exit,
    Sound {
        volume: u8,
        muted: bool,
    },
}
