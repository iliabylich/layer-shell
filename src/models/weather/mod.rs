use crate::models::Event;
use anyhow::Result;
use chrono::{NaiveDate, NaiveDateTime};
use reqwest::Client;
use tokio::sync::mpsc::Sender;

mod api;
use api::{get_weather, Response};
mod code;
use code::WeatherCode;

pub(crate) async fn spawn(tx: Sender<Event>) {
    if let Err(err) = try_spawn(tx).await {
        log::error!("Weather model error: {}\n{}", err, err.backtrace());
    }
}

async fn try_spawn(tx: Sender<Event>) -> Result<()> {
    let client = Client::new();

    loop {
        match get_weather(&client).await {
            Ok(response) => {
                let (current, forecast) = map_response_to_events(response);
                if let Err(err) = tx.send(current).await {
                    log::error!("Failed to send WeatherCurrent event: {}", err);
                }

                if let Err(err) = tx.send(forecast).await {
                    log::error!("Failed to send WeatherForecast event: {}", err);
                }
            }
            Err(err) => {
                log::error!("Failed to get weather: {}", err);
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(100)).await;
    }
}

fn map_response_to_events(
    Response {
        current,
        hourly,
        daily,
    }: Response,
) -> (Event, Event) {
    let current = format!(
        "{} {}",
        current.temperature_2m,
        WeatherCode::from(current.weather_code)
    );

    let now = chrono::Local::now().naive_local();
    let today = chrono::Local::now().date_naive();

    let hourly = hourly
        .temperature_2m
        .into_iter()
        .zip(hourly.weather_code)
        .zip(hourly.time)
        .map(|((temp, code), time)| {
            (
                temp,
                WeatherCode::from(code),
                NaiveDateTime::parse_from_str(&time, "%Y-%m-%dT%H:%M").unwrap(),
            )
        })
        .filter(|(_, _, time)| *time > now)
        .take(10)
        .map(|(temp, code, time)| {
            (
                format!("{}' {} {}", time.format("%H"), temp, code.icon()),
                code.to_string(),
            )
        })
        .collect::<Vec<_>>();

    let daily = daily
        .temperature_2m_min
        .into_iter()
        .zip(daily.temperature_2m_max)
        .zip(daily.weather_code)
        .zip(daily.time)
        .map(|(((min, max), code), time)| {
            (
                min,
                max,
                WeatherCode::from(code),
                NaiveDate::parse_from_str(&time, "%Y-%m-%d").unwrap(),
            )
        })
        .filter(|(_, _, _, date)| *date > today)
        .take(6)
        .map(|(min, max, code, date)| {
            (
                format!(
                    "{}: {:.1} - {:.1} {}",
                    date.format("%m-%d"),
                    min,
                    max,
                    code.icon()
                ),
                code.to_string(),
            )
        })
        .collect::<Vec<_>>();

    (
        Event::WeatherCurrent(current),
        Event::WeatherForecast { hourly, daily },
    )
}
