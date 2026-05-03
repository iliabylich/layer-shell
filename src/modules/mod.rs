mod caps_lock;
mod clock;
mod control;
mod cpu;
mod location;
mod memory;
mod network;
mod niri;
mod session_dbus;
mod sound;
mod system_dbus;
mod timer;
mod tray;
mod weather;

pub(crate) use caps_lock::CapsLock;
pub(crate) use clock::Clock;
pub(crate) use control::{Control, ControlRequest};
pub(crate) use cpu::CPU;
pub(crate) use location::Location;
pub(crate) use memory::Memory;
pub(crate) use network::Network;
pub(crate) use niri::Niri;
pub(crate) use session_dbus::SessionDBus;
pub(crate) use sound::Sound;
pub(crate) use system_dbus::SystemDBus;
pub(crate) use timer::Timer;
pub(crate) use tray::Tray;
pub(crate) use weather::Weather;

pub use tray::{TrayIcon, TrayIconPixmap, TrayItem};
pub use weather::{
    DAILY_WEATHER_FORECAST_LENGTH, HOURLY_WEATHER_FORECAST_LENGTH, WeatherCode, WeatherOnDay,
    WeatherOnHour,
};

use crate::sansio::{Satisfy, Wants};
use anyhow::Result;

pub(crate) trait Module {
    type Output;

    fn wants(&mut self) -> Result<Option<Wants>>;
    fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Self::Output;
}
