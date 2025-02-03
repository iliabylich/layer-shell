use crate::scheduler::Actor;
use anyhow::Result;
use std::{ops::ControlFlow, time::Duration};

mod client;
mod code;
mod mapper;

pub use code::WeatherCode;

#[derive(Debug)]
pub(crate) struct Weather;

impl Actor for Weather {
    fn name() -> &'static str {
        "Weather"
    }

    fn start() -> Result<Box<dyn Actor>> {
        Ok(Box::new(Weather))
    }

    fn tick(&mut self) -> Result<ControlFlow<(), Duration>> {
        let res = client::get_weather()?;

        let event = mapper::map_current(res.current);
        event.emit();

        let event = mapper::map_forecast(res.hourly, res.daily)?;
        event.emit();
        Ok(ControlFlow::Continue(Duration::from_secs(120)))
    }

    fn exec(&mut self, _: &crate::Command) -> Result<ControlFlow<()>> {
        Ok(ControlFlow::Break(()))
    }
}
