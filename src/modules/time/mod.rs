use crate::{scheduler::Actor, Event};
use anyhow::{Context as _, Result};
use std::{ops::ControlFlow, sync::mpsc::Sender, time::Duration};

#[derive(Debug)]
pub(crate) struct Time {
    tx: Sender<Event>,
}

impl Actor for Time {
    fn name() -> &'static str {
        "Time"
    }

    fn start(tx: Sender<Event>) -> Result<Box<dyn Actor>> {
        Ok(Box::new(Time { tx }))
    }

    fn tick(&mut self) -> Result<ControlFlow<(), Duration>> {
        let now = chrono::Local::now();
        let event = Event::Time {
            time: now.format("%H:%M:%S").to_string().into(),
            date: now.format("%Y %B %e").to_string().into(),
        };
        self.tx.send(event).context("failed to send event")?;
        Ok(ControlFlow::Continue(Duration::from_secs(1)))
    }

    fn exec(&mut self, _: &crate::Command) -> Result<ControlFlow<()>> {
        Ok(ControlFlow::Break(()))
    }
}
