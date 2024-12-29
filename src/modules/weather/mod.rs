use async_stream::stream;
use futures::Stream;
use reqwest::Client;

mod client;
mod code;
mod mapper;

pub use code::{
    WeatherCode, Drizzle, Fog, FreezingDrizzle, FreezingRain, Rain, RainShowers, SnowFall, SnowShowers,
    ThunderstormWithHail,
};

use crate::Event;

pub fn connect() -> impl Stream<Item = Event> {
    stream! {
        let client = Client::new();

        loop {
            match client::get_weather(&client).await {
                Ok(res) => {
                    let current = mapper::map_current(res.current);
                    yield current;
                    match mapper::map_forecast(res.hourly, res.daily) {
                        Ok(forecast) => yield forecast,
                        Err(err) => log::error!("{:?}", err),
                    }
                }
                Err(err) => log::error!("{:?}", err),
            }

            tokio::time::sleep(std::time::Duration::from_secs(100)).await;
        }
    }
}
