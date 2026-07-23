mod clock;
mod control;
mod cpu;
mod kb_mod;
mod memory;
mod niri;
mod nm;
mod pw;
mod timer;
mod tray;
mod weather;

pub use clock::Clock;
pub use control::Control;
pub use cpu::Cpu;
pub use kb_mod::KbMod;
pub use memory::Memory;
pub use niri::Niri;
pub use nm::NM;
pub use pw::PW;
pub use timer::Timer;
pub use tray::Tray;
pub use weather::Weather;

pub use cpu::MAX_CPU_COUNT;
pub use kb_mod::KbModKind;
pub use tray::{MaybeRootTrayElement, TrayElement, TrayLabel, TrayMenu};
pub use weather::{
    DAILY_WEATHER_FORECAST_LENGTH, HOURLY_WEATHER_FORECAST_LENGTH, WeatherCode, WeatherOnDay,
    WeatherOnHour,
};

use crate::{emitter::Emitter, module_id::ModuleId};

pub trait Module {
    const MODULE_ID: ModuleId;

    fn read(&mut self, emitter: Emitter) -> Result<(), ()>;
    fn id(&self) -> ModuleId {
        Self::MODULE_ID
    }
}

pub trait OptionModuleExt<T>
where
    T: Module,
{
    fn id(&self) -> ModuleId;
}

impl<T> OptionModuleExt<T> for Option<T>
where
    T: Module,
{
    fn id(&self) -> ModuleId {
        T::MODULE_ID
    }
}
