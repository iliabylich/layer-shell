use crate::{
    ffi::{FFIArray, FFIString},
    modules::{HyprlandWorkspace, TrayIcon, TrayItem, WeatherCode, WeatherOnDay, WeatherOnHour},
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
        time: FFIString,
    },
    Workspaces {
        workspaces: FFIArray<HyprlandWorkspace>,
    },
    Language {
        lang: FFIString,
    },
    Weather {
        temperature: f32,
        code: WeatherCode,
        hourly_forecast: FFIArray<WeatherOnHour>,
        daily_forecast: FFIArray<WeatherOnDay>,
    },
    NetworkSsid {
        ssid: FFIString,
    },
    NetworkStrength {
        strength: u8,
    },
    UploadSpeed {
        speed: FFIString,
    },
    DownloadSpeed {
        speed: FFIString,
    },
    TrayAppAdded {
        service: FFIString,
        items: FFIArray<TrayItem>,
        icon: TrayIcon,
    },
    TrayAppIconUpdated {
        service: FFIString,
        icon: TrayIcon,
    },
    TrayAppMenuUpdated {
        service: FFIString,
        items: FFIArray<TrayItem>,
    },
    TrayAppRemoved {
        service: FFIString,
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
