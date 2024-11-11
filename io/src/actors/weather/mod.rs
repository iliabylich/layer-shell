use crate::{
    event::{WeatherOnDay, WeatherOnHour},
    Event,
};
use anyhow::{Context, Result};
use chrono::{NaiveDate, NaiveDateTime};
use reqwest::Client;
use std::sync::mpsc::Sender;

mod api;
use api::{get_weather, CurrentResponse, DailyResponse, HourlyResponse, Response};
mod code;
pub use code::{
    Drizzle, Fog, FreezingDrizzle, FreezingRain, Rain, RainShowers, SnowFall, SnowShowers,
    ThunderstormWithHail, WeatherCode,
};

pub(crate) async fn spawn(tx: Sender<Event>) {
    if let Err(err) = try_spawn(tx).await {
        log::error!("Weather model error: {}\n{}", err, err.backtrace());
    }
}

async fn try_spawn(tx: Sender<Event>) -> Result<()> {
    let client = Client::new();

    loop {
        match get_weather(&client).await {
            Ok(response) => match map_response_to_events(response) {
                Ok((current, forecast)) => {
                    if let Err(err) = tx.send(current) {
                        log::error!("Failed to send WeatherCurrent event: {}", err);
                    }

                    if let Err(err) = tx.send(forecast) {
                        log::error!("Failed to send WeatherForecast event: {}", err);
                    }
                }
                Err(err) => {
                    log::error!("Failed to map weather: {}\n{}", err, err.backtrace());
                }
            },
            Err(err) => {
                log::error!("Failed to get weather: {}\n{}", err, err.backtrace());
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
) -> Result<(Event, Event)> {
    let hourly = map_hourly(hourly)?;
    let daily = map_daily(daily)?;

    Ok((
        map_current(current),
        Event::WeatherForecast { hourly, daily },
    ))
}

fn map_current(current: CurrentResponse) -> Event {
    Event::WeatherCurrent {
        temperature: current.temperature_2m,
        code: WeatherCode::from(current.weather_code),
    }
}

fn map_hourly(
    HourlyResponse {
        time,
        temperature_2m,
        weather_code,
    }: HourlyResponse,
) -> Result<Vec<WeatherOnHour>> {
    let now = chrono::Local::now().naive_local();

    let mut hourly = vec![];
    for ((temp, code), time) in temperature_2m.into_iter().zip(weather_code).zip(time) {
        let code = WeatherCode::from(code);
        let time = NaiveDateTime::parse_from_str(&time, "%Y-%m-%dT%H:%M")
            .context("invalid date format")?;
        if time > now {
            hourly.push(WeatherOnHour {
                hour: time.format("%H").to_string(),
                temperature: temp,
                code,
            });
        }
        if hourly.len() == 10 {
            break;
        }
    }

    assert_eq!(hourly.len(), 10);
    Ok(hourly)
}

fn map_daily(
    DailyResponse {
        time,
        temperature_2m_min,
        temperature_2m_max,
        weather_code,
    }: DailyResponse,
) -> Result<Vec<WeatherOnDay>> {
    let today = chrono::Local::now().date_naive();

    let mut daily = vec![];
    for (((min, max), code), time) in temperature_2m_min
        .into_iter()
        .zip(temperature_2m_max)
        .zip(weather_code)
        .zip(time)
    {
        let code = WeatherCode::from(code);
        let date = NaiveDate::parse_from_str(&time, "%Y-%m-%d").context("invalid date format")?;
        if date > today {
            daily.push(WeatherOnDay {
                day: date.format("%m-%d").to_string(),
                temperature: min..max,
                code,
            });
        }
        if daily.len() == 6 {
            break;
        }
    }

    assert_eq!(daily.len(), 6);
    Ok(daily)
}
