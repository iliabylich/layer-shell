use crate::{scheduler::Module, Event};
use anyhow::Result;
use std::any::Any;

pub(crate) struct Time;

impl Module for Time {
    const NAME: &str = "Time";
    const INTERVAL: Option<u64> = Some(1_000);

    fn start() -> Result<Box<dyn Any + Send + 'static>> {
        Ok(Box::new(0))
    }

    fn tick(_: &mut Box<dyn Any + Send + 'static>) -> Result<()> {
        let now = chrono::Local::now();
        let event = Event::Time {
            time: now.format("%H:%M:%S").to_string().into(),
            date: now.format("%Y %B %e").to_string().into(),
        };
        event.emit();
        Ok(())
    }
}
