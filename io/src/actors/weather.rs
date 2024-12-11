use crate::Event;
use anyhow::Result;
use futures::{pin_mut, StreamExt};
use std::sync::mpsc::Sender;

pub(crate) async fn spawn(tx: Sender<Event>) {
    if let Err(err) = try_spawn(tx).await {
        log::error!("{:?}", err);
    }
}

async fn try_spawn(tx: Sender<Event>) -> Result<()> {
    let weather_stream = layer_shell_weather::connect();
    pin_mut!(weather_stream);

    while let Some(event) = weather_stream.next().await {
        if let Err(err) = tx.send(Event::from(event)) {
            log::error!("Failed to send event: {:?}", err);
        }
    }

    Ok(())
}

impl From<layer_shell_weather::Event> for Event {
    fn from(event: layer_shell_weather::Event) -> Self {
        match event {
            layer_shell_weather::Event::CurrentWeather(e) => Self::CurrentWeather(e),
            layer_shell_weather::Event::ForecastWeather(e) => Self::ForecastWeather(e),
        }
    }
}
