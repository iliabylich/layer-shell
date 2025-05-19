use crate::{channel::EventSender, fd_id::FdId};
use anyhow::Result;
use mio::Token;
use std::os::fd::AsRawFd;

pub(crate) mod clock;
pub(crate) mod control;
pub(crate) mod cpu;
pub(crate) mod hyprland;
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

    fn new(tx: &EventSender) -> Result<Self>
    where
        Self: Sized;

    fn try_new(tx: &EventSender) -> Option<Self>
    where
        Self: Sized,
    {
        match Self::new(tx) {
            Ok(module) => Some(module),
            Err(err) => {
                log::error!("[{}] {err:?}", Self::NAME);
                None
            }
        }
    }

    fn read_events(&mut self) -> Result<Self::ReadOutput>;
}

pub(crate) trait MaybeModule {
    type ReadOutput;

    fn read_events_or_unregister(&mut self, poll: &crate::poll::Poll) -> Option<Self::ReadOutput>;
}

impl<T> MaybeModule for Option<T>
where
    T: Module,
{
    type ReadOutput = T::ReadOutput;

    fn read_events_or_unregister(&mut self, poll: &crate::poll::Poll) -> Option<Self::ReadOutput> {
        match self.as_mut() {
            Some(reader) => match reader.read_events() {
                Ok(output) => Some(output),
                Err(err) => {
                    log::error!("[{}] {err:?}", T::NAME);
                    poll.remove_reader(reader);
                    None
                }
            },
            None => {
                log::error!("[{}] unexpected epoll event", T::NAME);
                None
            }
        }
    }
}

pub(crate) trait TickingModule {
    const NAME: &str;

    fn tick(&mut self) -> Result<()>;
}

pub(crate) trait MaybeTickingModule {
    fn tick(&mut self);
}

impl<T> MaybeTickingModule for Option<T>
where
    T: TickingModule,
{
    fn tick(&mut self) {
        if let Some(callable) = self.as_mut() {
            if let Err(err) = callable.tick() {
                log::error!("module {} returned an error: {err:?}", T::NAME);
                *self = None;
            }
        }
    }
}
