use std::time::Duration;

use crate::{
    hyprctl,
    scheduler::{Module, RepeatingModule},
    Command,
};
use anyhow::Result;

pub(crate) struct Session;

impl Module for Session {
    const NAME: &str = "Session";

    fn start() -> Result<Option<Box<dyn RepeatingModule>>> {
        Ok(Some(Box::new(Self)))
    }
}

impl RepeatingModule for Session {
    fn tick(&mut self) -> Result<Duration> {
        Ok(Duration::from_secs(100_000))
    }

    fn exec(&mut self, cmd: &Command) -> Result<()> {
        match cmd {
            Command::Lock => hyprctl::dispatch("exec hyprlock")?,
            Command::Reboot => hyprctl::dispatch("exec systemctl reboot")?,
            Command::Shutdown => hyprctl::dispatch("exec systemctl poweroff")?,
            Command::Logout => hyprctl::dispatch("exit")?,

            _ => {}
        }

        Ok(())
    }
}
