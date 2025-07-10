use clock::ClockEvent;
use control::ControlEvent;
use cpu::CpuUsageEvent;
use hyprland::{HyprlandEvent, LanguageEvent, WorkspacesEvent};
use memory::MemoryEvent;
use network::{
    DownloadSpeedEvent, NetworkEvent, NetworkListEvent, NetworkSsidEvent, NetworkStrengthEvent,
    UploadSpeedEvent,
};
use sound::{InitialSoundEvent, MuteChangedEvent, SoundEvent, VolumeChangedEvent};
use tray::{
    TrayAppAddedEvent, TrayAppIconUpdatedEvent, TrayAppMenuUpdatedEvent, TrayAppRemovedEvent,
    TrayEvent,
};
use weather::{
    CurrentWeatherEvent, DailyWeatherForecastEvent, HourlyWeatherForecastEvent, WeatherEvent,
};

#[derive(Debug)]
#[repr(C)]
#[must_use]
pub enum Event {
    Memory(MemoryEvent),
    CpuUsage(CpuUsageEvent),
    Clock(ClockEvent),
    Workspaces(WorkspacesEvent),
    Language(LanguageEvent),
    CurrentWeather(CurrentWeatherEvent),
    HourlyWeatherForecast(HourlyWeatherForecastEvent),
    DailyWeatherForecast(DailyWeatherForecastEvent),
    NetworkSsid(NetworkSsidEvent),
    NetworkStrength(NetworkStrengthEvent),
    UploadSpeed(UploadSpeedEvent),
    DownloadSpeed(DownloadSpeedEvent),
    NetworkList(NetworkListEvent),
    TrayAppAdded(TrayAppAddedEvent),
    TrayAppIconUpdated(TrayAppIconUpdatedEvent),
    TrayAppMenuUpdated(TrayAppMenuUpdatedEvent),
    TrayAppRemoved(TrayAppRemovedEvent),
    ToggleSessionScreen,
    ReloadStyles,
    Exit,
    VolumeChanged(VolumeChangedEvent),
    MuteChanged(MuteChangedEvent),
    InitialSound(InitialSoundEvent),
}

impl From<HyprlandEvent> for Event {
    fn from(event: HyprlandEvent) -> Self {
        match event {
            HyprlandEvent::Workspaces(e) => Self::Workspaces(e),
            HyprlandEvent::Language(e) => Self::Language(e),
        }
    }
}

impl From<CpuUsageEvent> for Event {
    fn from(event: CpuUsageEvent) -> Self {
        Self::CpuUsage(event)
    }
}

impl From<MemoryEvent> for Event {
    fn from(event: MemoryEvent) -> Self {
        Self::Memory(event)
    }
}

impl From<ClockEvent> for Event {
    fn from(event: ClockEvent) -> Self {
        Self::Clock(event)
    }
}

impl From<ControlEvent> for Event {
    fn from(event: ControlEvent) -> Self {
        match event {
            ControlEvent::ToggleSessionScreen => Self::ToggleSessionScreen,
            ControlEvent::ReloadStyles => Self::ReloadStyles,
            ControlEvent::Exit => Self::Exit,
        }
    }
}

impl From<NetworkEvent> for Event {
    fn from(event: NetworkEvent) -> Self {
        match event {
            NetworkEvent::Ssid(e) => Self::NetworkSsid(e),
            NetworkEvent::Strength(e) => Self::NetworkStrength(e),
            NetworkEvent::UploadSpeed(e) => Self::UploadSpeed(e),
            NetworkEvent::DownloadSpeed(e) => Self::DownloadSpeed(e),
            NetworkEvent::NetworkList(e) => Self::NetworkList(e),
        }
    }
}

impl From<WeatherEvent> for Event {
    fn from(event: WeatherEvent) -> Self {
        match event {
            WeatherEvent::CurrentWeather(e) => Self::CurrentWeather(e),
            WeatherEvent::HourlyWeatherForecast(e) => Self::HourlyWeatherForecast(e),
            WeatherEvent::DailyWeatherForecast(e) => Self::DailyWeatherForecast(e),
        }
    }
}

impl From<TrayEvent> for Event {
    fn from(event: TrayEvent) -> Self {
        match event {
            TrayEvent::AppAdded(e) => Self::TrayAppAdded(e),
            TrayEvent::AppRemoved(e) => Self::TrayAppRemoved(e),
            TrayEvent::AppIconUpdated(e) => Self::TrayAppIconUpdated(e),
            TrayEvent::AppMenuUpdated(e) => Self::TrayAppMenuUpdated(e),
        }
    }
}

impl From<SoundEvent> for Event {
    fn from(event: SoundEvent) -> Self {
        match event {
            SoundEvent::MuteChangedEvent(e) => Self::MuteChanged(e),
            SoundEvent::VolumeChangedEvent(e) => Self::VolumeChanged(e),
            SoundEvent::InitialSoundEvent(e) => Self::InitialSound(e),
        }
    }
}
