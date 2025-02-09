use crate::{scheduler::Actor, Event};
use anyhow::{Context as _, Result};
use std::{ops::ControlFlow, sync::mpsc::Sender, time::Duration};

mod client;
mod code;
mod mapper;

pub use code::WeatherCode;

#[derive(Debug)]
pub(crate) struct Weather {
    tx: Sender<Event>,
}

impl Actor for Weather {
    fn name() -> &'static str {
        "Weather"
    }

    fn start(tx: Sender<Event>) -> Result<Box<dyn Actor>> {
        Ok(Box::new(Weather { tx }))
    }

    fn tick(&mut self) -> Result<ControlFlow<(), Duration>> {
        let res = client::get_weather()?;

        let (current, forecast) = mapper::map(res)?;
        self.tx
            .send(current)
            .context("failed to send current weather event")?;
        self.tx
            .send(forecast)
            .context("failed to send current weather event")?;
        Ok(ControlFlow::Continue(Duration::from_secs(120)))
    }

    fn exec(&mut self, _: &crate::Command) -> Result<ControlFlow<()>> {
        Ok(ControlFlow::Break(()))
    }
}
