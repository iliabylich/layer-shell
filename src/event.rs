use crate::{
    ffi::{CArray, CString},
    modules::{WeatherCode, WeatherOnDay, WeatherOnHour},
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
        usage_per_core: CArray<u8>,
    },
    Clock {
        time: CString,
    },
    Workspaces {
        workspaces: CArray<Workspace>,
    },
    Language {
        lang: CString,
    },
    Weather {
        temperature: f32,
        code: WeatherCode,
        hourly_forecast: CArray<WeatherOnHour>,
        daily_forecast: CArray<WeatherOnDay>,
    },
    NetworkSsid {
        ssid: CString,
    },
    NetworkStrength {
        strength: u8,
    },
    UploadSpeed {
        speed: CString,
    },
    DownloadSpeed {
        speed: CString,
    },
    // TrayAppAdded(TrayAppAddedEvent),
    // TrayAppIconUpdated(TrayAppIconUpdatedEvent),
    // TrayAppMenuUpdated(TrayAppMenuUpdatedEvent),
    // TrayAppRemoved(TrayAppRemovedEvent),
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

#[derive(Debug)]
#[repr(C)]
pub struct Workspace {
    pub visible: bool,
    pub active: bool,
}
