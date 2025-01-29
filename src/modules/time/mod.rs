use crate::{
    scheduler::{Module, RepeatingModule},
    Event,
};
use anyhow::Result;
use std::time::Duration;

pub(crate) struct Time;

impl Module for Time {
    const NAME: &str = "Time";

    fn start() -> Result<Option<Box<dyn RepeatingModule>>> {
        Ok(Some(Box::new(Time)))
    }
}

impl RepeatingModule for Time {
    fn tick(&mut self) -> Result<Duration> {
        let now = chrono::Local::now();
        let event = Event::Time {
            time: now.format("%H:%M:%S").to_string().into(),
            date: now.format("%Y %B %e").to_string().into(),
        };
        event.emit();

        Ok(Duration::from_secs(1))
    }

    fn exec(&mut self, _: &crate::Command) -> Result<()> {
        Ok(())
    }
}
