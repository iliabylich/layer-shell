use crate::{
    ffi::{CArray, CString},
    weather::WeatherCode,
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
    // NetworkSsid(NetworkSsidEvent),
    // NetworkStrength(NetworkStrengthEvent),
    // UploadSpeed(UploadSpeedEvent),
    // DownloadSpeed(DownloadSpeedEvent),
    // NetworkList(NetworkListEvent),
    // TrayAppAdded(TrayAppAddedEvent),
    // TrayAppIconUpdated(TrayAppIconUpdatedEvent),
    // TrayAppMenuUpdated(TrayAppMenuUpdatedEvent),
    // TrayAppRemoved(TrayAppRemovedEvent),
    // ToggleSessionScreen,
    // ReloadStyles,
    // CapsLockToggled(ControlCapsLockToggledEvent),
    // Exit,
    // VolumeChanged(VolumeChangedEvent),
    // MuteChanged(MuteChangedEvent),
    // InitialSound(InitialSoundEvent),
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
pub struct Workspace {
    pub visible: bool,
    pub active: bool,
}
