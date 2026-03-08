mod clock;
mod control;
mod cpu;
mod hyprland;
mod location;
mod memory;
mod network;
mod session_dbus;
mod sound;
mod system_dbus;
mod tray;
mod weather;

pub(crate) use clock::Clock;
pub(crate) use control::{Control, ControlRequest};
pub(crate) use cpu::CPU;
pub(crate) use hyprland::{Hyprland, HyprlandReader, HyprlandWriter};
pub(crate) use location::Location;
pub(crate) use memory::Memory;
pub(crate) use network::Network;
pub(crate) use session_dbus::SessionDBus;
pub(crate) use sound::Sound;
pub(crate) use system_dbus::SystemDBus;
pub(crate) use tray::Tray;
pub(crate) use weather::Weather;

pub use hyprland::HyprlandWorkspace;
pub use tray::{TrayIcon, TrayIconPixmap, TrayItem};
pub use weather::{WeatherCode, WeatherOnDay, WeatherOnHour};

use crate::{
    Event,
    dbus::Message,
    sansio::{Satisfy, Wants},
    user_data::ModuleId,
};
use anyhow::Result;
use std::convert::Infallible;

pub(crate) trait DBusQueued {
    fn enqueue(&mut self, message: &mut Message);
}

pub(crate) trait Module {
    type Input;
    type Output;
    type Error: std::fmt::Debug;

    const MODULE_ID: ModuleId;

    fn new(input: Self::Input) -> Self
    where
        Self: Sized;
    fn wants(&mut self) -> Wants;
    fn satisfy(
        &mut self,
        satisfy: Satisfy,
        res: i32,
        events: &mut Vec<Event>,
    ) -> Result<Option<Self::Output>, Self::Error>;
    fn tick(&mut self, tick: u64);
}

// pub(crate) struct Falliable<T>
// where
//     T: Module,
// {
//     pub(crate) inner: Option<T>,
// }

// impl<T> Falliable<T>
// where
//     T: Module,
// {
//     pub(crate) fn none() -> Self {
//         Self { inner: None }
//     }

//     pub(crate) fn some(inner: T) -> Self {
//         Self { inner: Some(inner) }
//     }

//     pub(crate) fn start_with(&mut self, input: T::Input) {
//         self.inner = Some(T::new(input))
//     }
// }

impl<T> Module for Option<T>
where
    T: Module,
{
    type Input = T::Input;
    type Output = T::Output;
    type Error = Infallible;

    const MODULE_ID: ModuleId = T::MODULE_ID;

    fn new(input: Self::Input) -> Self {
        Some(T::new(input))
    }

    fn wants(&mut self) -> Wants {
        let Some(inner) = self else {
            return Wants::Nothing;
        };

        inner.wants()
    }

    fn satisfy(
        &mut self,
        satisfy: Satisfy,
        res: i32,
        events: &mut Vec<Event>,
    ) -> Result<Option<Self::Output>, Self::Error> {
        let Some(inner) = self else {
            return Ok(None);
        };

        match inner.satisfy(satisfy, res, events) {
            Ok(out) => Ok(out),
            Err(err) => {
                log::error!("Module {:?} has crashed: {err:?}", Self::MODULE_ID);
                *self = None;
                Ok(None)
            }
        }
    }

    fn tick(&mut self, tick: u64) {
        if let Some(inner) = self {
            inner.tick(tick);
        }
    }
}
