use anyhow::Result;

mod client;
mod code;
mod mapper;

pub use code::WeatherCode;

pub(crate) fn tick() -> Result<()> {
    let res = client::get_weather()?;

    let event = mapper::map_current(res.current);
    event.emit();

    let event = mapper::map_forecast(res.hourly, res.daily)?;
    event.emit();

    Ok(())
}
