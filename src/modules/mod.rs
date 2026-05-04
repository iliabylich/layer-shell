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

use crate::{
    sansio::{Satisfy, Wants},
    user_data::ModuleId,
};
use anyhow::Result;

pub(crate) trait FallibleModule {
    const MODULE_ID: ModuleId;
    type Output;

    fn try_wants(&mut self) -> Result<Option<Wants>>;
    fn try_satisfy(&mut self, satisfy: Satisfy, res: i32) -> Result<Option<Self::Output>>;
    fn try_tick(&mut self, _tick: u64) -> Result<()> {
        Ok(())
    }
}

pub(crate) struct InfallibleModule<M> {
    module: Option<M>,
}

impl<M: FallibleModule> InfallibleModule<M> {
    const NAME: &str = M::MODULE_ID.as_str();

    pub(crate) const fn new(module: M) -> Self {
        Self {
            module: Some(module),
        }
    }

    pub(crate) fn wants(&mut self) -> Option<Wants> {
        match self.module.as_mut()?.try_wants() {
            Ok(wants) => wants,
            Err(err) => {
                log::error!(target: Self::NAME, "{err:?}");
                None
            }
        }
    }

    pub(crate) fn satisfy(&mut self, satisfy: Satisfy, res: i32) -> Option<M::Output> {
        match self.module.as_mut()?.try_satisfy(satisfy, res) {
            Ok(output) => output,
            Err(err) => {
                log::error!(target: Self::NAME, "crash, stopping. satisfy={satisfy:?}, res={res}, err: {err:?}");
                self.module = None;
                None
            }
        }
    }

    pub(crate) const fn inner(&mut self) -> Option<&mut M> {
        self.module.as_mut()
    }

    pub(crate) fn tick(&mut self, tick: u64) {
        let Some(module) = self.module.as_mut() else {
            return;
        };

        if let Err(err) = module.try_tick(tick) {
            log::error!(target: Self::NAME, "crash, stopping. err: {err:?}");
            self.module = None;
        }
    }

    #[expect(clippy::unused_self)]
    pub(crate) const fn module_id(&self) -> ModuleId {
        M::MODULE_ID
    }
}
