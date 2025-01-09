use std::sync::mpsc::{Receiver, Sender};

use crate::ffi::{CArray, CString};
use crate::global;
use crate::modules::weather::WeatherCode;

#[derive(Debug)]
#[repr(C)]
pub enum Event {
    Memory {
        used: f64,
        total: f64,
    },
    CpuUsage {
        usage_per_core: CArray<usize>,
    },
    Time {
        date: CString,
        time: CString,
    },
    Workspaces {
        ids: CArray<usize>,
        active_id: usize,
    },
    Language {
        lang: CString,
    },
    AppList {
        apps: CArray<App>,
    },
    Volume {
        volume: f32,
    },
    Mute {
        muted: bool,
    },
    CurrentWeather {
        temperature: f32,
        code: WeatherCode,
    },
    ForecastWeather {
        hourly: CArray<WeatherOnHour>,
        daily: CArray<WeatherOnDay>,
    },
    WiFiStatus {
        ssid: CString,
        strength: u8,
    },
    NetworkList {
        list: CArray<Network>,
    },
    ToggleLauncher,
    ToggleSessionScreen,
}

global!(SENDER, Sender<Event>);
global!(RECEIVER, Receiver<Event>);

impl Event {
    pub(crate) fn set_sender(sender: Sender<Event>) {
        SENDER::set(sender);
    }
    pub(crate) fn set_receiver(sender: Receiver<Event>) {
        RECEIVER::set(sender);
    }

    pub(crate) fn emit(self) {
        if let Err(err) = SENDER::get().send(self) {
            log::error!("failed to publish event: {:?}", err);
        }
    }

    pub(crate) fn try_recv() -> Option<Self> {
        RECEIVER::get().try_recv().ok()
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct App {
    pub name: CString,
    pub selected: bool,
    pub icon: AppIcon,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub enum AppIcon {
    IconPath(CString),
    IconName(CString),
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
pub struct Network {
    pub iface: CString,
    pub address: CString,
}
