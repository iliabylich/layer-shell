use crate::{hyprctl, scheduler::Actor, Command};
use anyhow::Result;
use std::{ops::ControlFlow, time::Duration};

#[derive(Debug)]
pub(crate) struct Session;

impl Actor for Session {
    fn name() -> &'static str {
        "Session"
    }

    fn start() -> Result<Box<dyn Actor>> {
        Ok(Box::new(Self))
    }

    fn tick(&mut self) -> Result<ControlFlow<(), Duration>> {
        Ok(ControlFlow::Break(()))
    }

    fn exec(&mut self, cmd: &Command) -> Result<ControlFlow<()>> {
        match cmd {
            Command::Lock => hyprctl::dispatch("exec hyprlock")?,
            Command::Reboot => hyprctl::dispatch("exec systemctl reboot")?,
            Command::Shutdown => hyprctl::dispatch("exec systemctl poweroff")?,
            Command::Logout => hyprctl::dispatch("exit")?,

            _ => {}
        }

        Ok(ControlFlow::Continue(()))
    }
}
