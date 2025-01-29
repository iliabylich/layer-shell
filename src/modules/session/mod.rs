use std::time::Duration;

use crate::{
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
            Command::Lock => crate::command::lock()?,
            Command::Reboot => crate::command::reboot()?,
            Command::Shutdown => crate::command::shutdown()?,
            Command::Logout => crate::command::logout()?,

            _ => {}
        }

        Ok(())
    }
}
