use crate::{
    ffi::FFIArray,
    modules::{
        DAILY_WEATHER_FORECAST_LENGTH, HOURLY_WEATHER_FORECAST_LENGTH, HyprlandWorkspace, TrayIcon,
        TrayItem, WeatherCode, WeatherOnDay, WeatherOnHour,
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
    Clock {
        unix_seconds: i64,
    },
    Workspaces {
        workspaces: FFIArray<HyprlandWorkspace>,
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
    NetworkSsid {
        ssid: StringRef,
    },
    NetworkStrength {
        strength: u8,
    },
    UploadSpeed {
        bytes_per_sec: u64,
    },
    DownloadSpeed {
        bytes_per_sec: u64,
    },
    TrayAppAdded {
        service: StringRef,
        items: FFIArray<TrayItem>,
        icon: TrayIcon,
    },
    TrayAppIconUpdated {
        service: StringRef,
        icon: TrayIcon,
    },
    TrayAppMenuUpdated {
        service: StringRef,
        items: FFIArray<TrayItem>,
    },
    TrayAppRemoved {
        service: StringRef,
    },
    ToggleSessionScreen,
    ReloadStyles,
    CapsLockToggled {
        enabled: bool,
    },
    Exit,
    VolumeChanged {
        volume: u32,
    },
    MuteChanged {
        muted: bool,
    },
    InitialSound {
        volume: u32,
        muted: bool,
    },
}
