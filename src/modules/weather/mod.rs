use std::any::Any;

use crate::scheduler::Module;
use anyhow::Result;

mod client;
mod code;
mod mapper;

pub use code::WeatherCode;

pub(crate) struct Weather;

impl Module for Weather {
    const NAME: &str = "Weather";
    const INTERVAL: Option<u64> = Some(120_000);

    fn start() -> Result<Box<dyn Any + Send + 'static>> {
        Ok(Box::new(0))
    }

    fn tick(_state: &mut Box<dyn Any + Send + 'static>) -> Result<()> {
        let res = client::get_weather()?;

        let event = mapper::map_current(res.current);
        event.emit();

        let event = mapper::map_forecast(res.hourly, res.daily)?;
        event.emit();

        Ok(())
    }
}
