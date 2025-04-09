use crate::{channel::EventSender, fd_id::FdId};
use anyhow::Result;
use mio::Token;
use std::os::fd::AsRawFd;

pub(crate) mod clock;
pub(crate) mod control;
pub(crate) mod cpu;
pub(crate) mod hyprland;
pub(crate) mod launcher;
pub(crate) mod memory;
pub(crate) mod network;
pub(crate) mod pipewire;
pub(crate) mod session;
pub(crate) mod tray;
pub(crate) mod weather;

pub(crate) trait Module: AsRawFd {
    const FD_ID: FdId;
    const TOKEN: Token = Self::FD_ID.token();
    const NAME: &str;

    type ReadOutput;

    fn new(tx: EventSender) -> Result<Self>
    where
        Self: Sized;

    fn read_events(&mut self) -> Result<Self::ReadOutput>;
}

pub(crate) trait TickingModule {
    const NAME: &str;

    fn tick(&mut self) -> Result<()>;
}
