use crate::scheduler::Module;
use anyhow::Result;

mod client;
mod code;
mod mapper;

pub use code::WeatherCode;

pub(crate) struct Weather;

impl Module for Weather {
    const NAME: &str = "Weather";

    fn start() -> Result<Option<(u64, fn() -> Result<()>)>> {
        Ok(Some((120_000, tick)))
    }
}

fn tick() -> Result<()> {
    let res = client::get_weather()?;

    let event = mapper::map_current(res.current);
    event.emit();

    let event = mapper::map_forecast(res.hourly, res.daily)?;
    event.emit();

    Ok(())
}
