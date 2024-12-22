use crate::Event;
use futures::{pin_mut, StreamExt};
use std::sync::mpsc::Sender;

pub(crate) async fn spawn(tx: Sender<Event>) {
    let weather_stream = layer_shell_weather::connect().map(|event| match event {
        layer_shell_weather::Event::CurrentWeather(current) => Event::CurrentWeather(current),
        layer_shell_weather::Event::ForecastWeather(forecast) => Event::ForecastWeather(forecast),
    });
    pin_mut!(weather_stream);

    while let Some(event) = weather_stream.next().await {
        if let Err(err) = tx.send(event) {
            log::error!("Failed to send event: {:?}", err);
        }
    }
}
