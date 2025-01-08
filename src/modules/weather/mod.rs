mod client;
mod code;
mod mapper;

pub use code::WeatherCode;

pub(crate) fn tick() {
    match client::get_weather() {
        Ok(res) => {
            let event = mapper::map_current(res.current);
            event.emit();

            match mapper::map_forecast(res.hourly, res.daily) {
                Ok(event) => event.emit(),
                Err(err) => log::error!("{:?}", err),
            }
        }
        Err(err) => log::error!("{:?}", err),
    }
}
