use crate::{scheduler::Actor, Event};
use anyhow::Result;
use std::{ops::ControlFlow, time::Duration};

#[derive(Debug)]
pub(crate) struct Time;

impl Actor for Time {
    fn name() -> &'static str {
        "Time"
    }

    fn start() -> Result<Box<dyn Actor>> {
        Ok(Box::new(Time))
    }

    fn tick(&mut self) -> Result<ControlFlow<(), Duration>> {
        let now = chrono::Local::now();
        let event = Event::Time {
            time: now.format("%H:%M:%S").to_string().into(),
            date: now.format("%Y %B %e").to_string().into(),
        };
        event.emit();
        Ok(ControlFlow::Continue(Duration::from_secs(1)))
    }

    fn exec(&mut self, _: &crate::Command) -> Result<ControlFlow<()>> {
        Ok(ControlFlow::Break(()))
    }
}
