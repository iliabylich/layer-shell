use crate::{scheduler::Module, Event};
use anyhow::Result;

pub(crate) struct Time;

impl Module for Time {
    const NAME: &str = "Time";

    fn start() -> Result<Option<(u64, fn() -> Result<()>)>> {
        Ok(Some((1_000, tick)))
    }
}

fn tick() -> Result<()> {
    let now = chrono::Local::now();
    let event = Event::Time {
        time: now.format("%H:%M:%S").to_string().into(),
        date: now.format("%Y %B %e").to_string().into(),
    };
    event.emit();
    Ok(())
}
