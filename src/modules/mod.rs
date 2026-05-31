mod clock;
mod control;
mod cpu;
mod kb_mod;
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

pub(crate) use clock::Clock;
pub(crate) use control::{Control, ControlRequest};
pub(crate) use cpu::CPU;
pub(crate) use kb_mod::KbMod;
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

pub use kb_mod::KbModKind;
pub use tray::{TrayIcon, TrayIconPixmap, TrayItem};
pub use weather::{
    DAILY_WEATHER_FORECAST_LENGTH, HOURLY_WEATHER_FORECAST_LENGTH, WeatherCode, WeatherOnDay,
    WeatherOnHour,
};

use crate::{
    sansio::{Satisfy, Wants},
    user_data::ModuleId,
};
use anyhow::Result;

pub(crate) trait FallibleModule {
    const MODULE_ID: ModuleId;
    type Output;

    fn wants(&mut self) -> Result<Option<Wants>>;
    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<Self::Output>>;
    fn try_tick(&mut self, _tick: u64) -> Result<()> {
        Ok(())
    }
}
